use std::{io::{self, Write}, rc::Rc, cell::RefCell};

use crate::parser::{Command, ParsedCmd};

use super::{
    asm_generator::{arithmetic, terminate, MemoryError, MemCmdWriter},
    label_generator::LabelGenerator, reg_mgr::{RegMgr, RegMgrError},
};

#[derive(thiserror::Error, Debug)]
pub enum CodeWriterError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("Temp register error: {0}")]
    Temp(#[from] RegMgrError),
    #[error("Memory manipulation asm error: {0}")]
    Memory(#[from] MemoryError)
}

pub struct CodeWriter<W: Write> {
    out_stream: W,
    label_generator: LabelGenerator,
    gen_purp_reg: Rc<RefCell<RegMgr>>,
    mem_cmd_writer: MemCmdWriter,
}

impl<'a, W: Write> CodeWriter<W> {
    pub fn new(out_stream: W) -> Result<Self, CodeWriterError> {
        let gen_purp_reg = Rc::new(RefCell::new(RegMgr::new(13,15)?));
        let mem_cmd_writer = MemCmdWriter::new("asm".to_owned(), gen_purp_reg.clone());
        Ok(Self {
            out_stream,
            label_generator: LabelGenerator::new("asm"),
            gen_purp_reg,
            mem_cmd_writer
        })
    }

    pub fn set_namespace(&mut self, namespace: &str) {
        self.mem_cmd_writer = MemCmdWriter::new(namespace.to_owned(), self.gen_purp_reg.clone());
        self.label_generator = LabelGenerator::new(namespace);
    }

    pub fn comment(&mut self, comment: &str) -> Result<(), CodeWriterError> {
        Ok(writeln!(self.out_stream, "//{}", comment)?)
    }

    pub fn write(&mut self, cmd: Command) -> Result<(), CodeWriterError> {
        self.comment(cmd.original())?;
        if let Some(asm) = self.cmd_to_asm(cmd.parsed().clone())? {
            for line in asm {
                writeln!(self.out_stream, "{}", line)?;
            }
        };
        Ok(())
    }

    fn cmd_to_asm(&mut self, cmd: ParsedCmd) -> Result<Option<Vec<String>>, CodeWriterError> {
        match cmd {
            ParsedCmd::Arithmetic(arr) => Ok(Some(arithmetic(arr, &mut self.label_generator))),
            ParsedCmd::Push(segment, idx) => Ok(Some(self.mem_cmd_writer.push_to_stack(segment, idx)?)),
            ParsedCmd::Pop(segment, idx) => Ok(Some(self.mem_cmd_writer.pop_stack_to(segment, idx)?)),
            ParsedCmd::Label(_) => todo!(),
            ParsedCmd::Goto(_) => todo!(),
            ParsedCmd::If(_) => todo!(),
            ParsedCmd::Function(_, _) => todo!(),
            ParsedCmd::Call(_, _) => todo!(),
            ParsedCmd::Return => todo!(),
            ParsedCmd::Noop => Ok(None),
            ParsedCmd::Terminate => Ok(Some(terminate(&mut self.label_generator))),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::parser::{Arithmetic, PushSegment, PopSegment};
    use test_case::test_case;

    use super::*;

    fn make_buff() -> Vec<u8> {
        vec![0; 15]
    }

    fn make_writer(buff: &mut Vec<u8>) -> CodeWriter<Cursor<&mut Vec<u8>>> {
        let c = Cursor::new(buff);
        CodeWriter::new(c).unwrap()
    }

    fn make_command(cmd: ParsedCmd) -> Command {
        Command::new("".to_owned(), cmd)
    }

    #[test_case(
        ParsedCmd::Push(PushSegment::Constant, 5),
        "//\n@5\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push constant to stack"
    )]
    #[test_case(
        ParsedCmd::Push(PushSegment::Argument, 0),
        "//\n@ARG\nD=M\nA=D\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push first argument to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Argument, 0), 
        "//\n@ARG\nD=M\n@R13\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n"; 
        "pop stack to first argument"
    )]
    #[test_case(
        ParsedCmd::Push(PushSegment::Argument, 1),
        "//\n@ARG\nD=M\n@1\nD=D+A\nA=D\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push second argument to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Argument, 1), 
        "//\n@ARG\nD=M\n@1\nD=D+A\n@R13\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n"; 
        "pop stack to second argument"
    )]
    #[test_case(
        ParsedCmd::Push(PushSegment::Static, 1),
        "//\n@ASM.1\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push first static to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Static, 1),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@ASM.1\nM=D\n"; 
        "pop stack to first static"
    )]
    #[test_case(
        ParsedCmd::Push(PushSegment::Static, 5),
        "//\n@ASM.5\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push fifth static to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Static, 5),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@ASM.5\nM=D\n"; 
        "pop stack to fifth static"
    )]
    #[test_case(
        ParsedCmd::Push(PushSegment::Pointer, 0),
        "//\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push pointer[0] to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Pointer, 0),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@THIS\nM=D\n"; 
        "pop stack to pointer[0]"
    )]
    #[test_case(
        ParsedCmd::Pop(PopSegment::Pointer, 1),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@THAT\nM=D\n"; 
        "pop stack to pointer[1]"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Add),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=M+D\n@SP\nM=M+1\n"; 
        "add"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Sub),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=M-D\n@SP\nM=M+1\n";
        "sub"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::And),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=M&D\n@SP\nM=M+1\n";
        "and"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Or),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=M|D\n@SP\nM=M+1\n";
        "or"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Neg),
        "//\n@SP\nM=M-1\nA=M\nM=-M\n@SP\nM=M+1\n";
        "neg"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Lt),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM_1\nD;JLT\nD=0\n@ASM_2\n0;JMP\n(ASM_1)\nD=-1\n(ASM_2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "lt"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Gt),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM_1\nD;JGT\nD=0\n@ASM_2\n0;JMP\n(ASM_1)\nD=-1\n(ASM_2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "gt"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Eq),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM_1\nD;JEQ\nD=0\n@ASM_2\n0;JMP\n(ASM_1)\nD=-1\n(ASM_2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "eq"
    )]
    #[test_case(
        ParsedCmd::Terminate, "//\n(ASM_1)\n@ASM_1\n0;JMP\n"; "terminate"
    )]
    fn test_asm_generation(cmd: ParsedCmd, expected_asm: &str) {
        let mut buff = make_buff();
        let mut writer = make_writer(&mut buff);
        let cmd = make_command(cmd);
        writer.write(cmd).unwrap();
        assert_eq!(expected_asm, std::str::from_utf8(&buff).unwrap())
    }
}
