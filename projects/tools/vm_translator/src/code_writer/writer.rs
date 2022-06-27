use std::{io::{self, Write}, rc::Rc, cell::RefCell};

use crate::parser::{Command, ParsedCmd, Flow};

use super::{
    asm_generator::{arithmetic, MemoryError, MemCmdWriter, flow, marker, FlowError},
    reg_mgr::{RegMgr, RegMgrError}, label_manager::LabelManager,
};

#[derive(thiserror::Error, Debug)]
pub enum CodeWriterError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("Temp register error: {0}")]
    Temp(#[from] RegMgrError),
    #[error("Memory manipulation asm error: {0}")]
    Memory(#[from] MemoryError),
    #[error("Control flow error: {0}")]
    Flow(#[from] FlowError)
}

pub struct CodeWriter<W: Write> {
    out_stream: W,
    label_manager: LabelManager,
    gen_purp_reg: Rc<RefCell<RegMgr>>,
    mem_cmd_writer: Rc<MemCmdWriter>,
}

impl<'a, W: Write> CodeWriter<W> {
    pub fn new(out_stream: W) -> Result<Self, CodeWriterError> {
        let gen_purp_reg = Rc::new(RefCell::new(RegMgr::new(13,15)?));
        let mem_cmd_writer = Rc::new(MemCmdWriter::new("asm".to_owned(), gen_purp_reg.clone()));
        let label_manager = LabelManager::new("asm");
        Ok(Self {
            out_stream,
            label_manager,
            gen_purp_reg,
            mem_cmd_writer
        })
    }

    pub fn init(&mut self) -> Result<(), CodeWriterError> {
        writeln!(self.out_stream, "@256\nD=A\n@SP\nM=D")?;
        self.write(Command::new("call Sys.init 0".to_string(), ParsedCmd::Flow(Flow::Call("Sys.init".to_string(), 0))))
    }

    pub fn set_namespace(&mut self, namespace: &str) {
        self.mem_cmd_writer = Rc::new(MemCmdWriter::new(namespace.to_owned(), self.gen_purp_reg.clone()));
        self.label_manager.set_filename(namespace);
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
            ParsedCmd::Arithmetic(arr) => Ok(Some(arithmetic(arr, &mut self.label_manager))),
            ParsedCmd::PushConstant(value) => Ok(Some(self.mem_cmd_writer.push_constant(value))),
            ParsedCmd::Push(segment, idx) => Ok(Some(self.mem_cmd_writer.push_to_stack(segment, idx)?)),
            ParsedCmd::Pop(segment, idx) => Ok(Some(self.mem_cmd_writer.pop_stack_to(segment, idx)?)),
            ParsedCmd::Flow(flow_cmd) => Ok(Some(flow(self.gen_purp_reg.clone(), self.mem_cmd_writer.clone())(flow_cmd, &mut self.label_manager)?)),
            ParsedCmd::Marker(marker_cmd) => Ok(Some(marker(marker_cmd, &mut self.label_manager))),
            ParsedCmd::Noop => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::parser::*;
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
        ParsedCmd::PushConstant(5),
        "//\n@5\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push constant to stack"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Argument, 0),
        "//\n@ARG\nA=M\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push first argument to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Argument, 0), 
        "//\n@ARG\nD=M\n@R13\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n"; 
        "pop stack to first argument"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Argument, 1),
        "//\n@ARG\nD=M\nA=D+1\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push second argument to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Argument, 1), 
        "//\n@ARG\nD=M+1\n@R13\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n"; 
        "pop stack to second argument"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Argument, 2),
        "//\n@ARG\nD=M\n@2\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "push third argument to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Argument, 2), 
        "//\n@ARG\nD=M\n@2\nD=D+A\n@R13\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n"; 
        "pop stack to third argument"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Static, 1),
        "//\n@ASM.1\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push first static to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Static, 1),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@ASM.1\nM=D\n"; 
        "pop stack to first static"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Static, 5),
        "//\n@ASM.5\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push fifth static to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Static, 5),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@ASM.5\nM=D\n"; 
        "pop stack to fifth static"
    )]
    #[test_case(
        ParsedCmd::Push(Segment::Pointer, 0),
        "//\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"; 
        "push pointer[0] to stack"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Pointer, 0),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@THIS\nM=D\n"; 
        "pop stack to pointer[0]"
    )]
    #[test_case(
        ParsedCmd::Pop(Segment::Pointer, 1),
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
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM.1\nD;JLT\nD=0\n@ASM.2\n0;JMP\n(ASM.1)\nD=-1\n(ASM.2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "lt"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Gt),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM.1\nD;JGT\nD=0\n@ASM.2\n0;JMP\n(ASM.1)\nD=-1\n(ASM.2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "gt"
    )]
    #[test_case(
        ParsedCmd::Arithmetic(Arithmetic::Eq),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@ASM.1\nD;JEQ\nD=0\n@ASM.2\n0;JMP\n(ASM.1)\nD=-1\n(ASM.2)\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "eq"
    )]
    #[test_case(
        ParsedCmd::Flow(Flow::Goto(Goto::Conditional, "test".to_owned())),
        "//\n@SP\nM=M-1\nA=M\nD=M\n@test\nD;JGT\nD;JLT\n";
        "if-goto"
    )]
    #[test_case(
        ParsedCmd::Flow(Flow::Goto(Goto::Direct, "test".to_owned())),
        "//\n@test\n0;JMP\n";
        "goto"
    )]
    #[test_case(
        ParsedCmd::Flow(Flow::Return),
        "//\n@LCL\nD=M\n@R13\nM=D\n@5\nA=D-A\nD=M\n@R14\nM=D\n@ARG\nD=M\n@R15\nM=D\n@SP\nM=M-1\nA=M\nD=M\n@R15\nA=M\nM=D\n@ARG\nD=M+1\n@SP\nM=D\n@R13\nA=M-1\nD=M\n@THAT\nM=D\n@R13\nD=M\n@2\nA=D-A\nD=M\n@THIS\nM=D\n@R13\nD=M\n@3\nA=D-A\nD=M\n@ARG\nM=D\n@R13\nD=M\n@4\nA=D-A\nD=M\n@LCL\nM=D\n@R14\nA=M\n0;JMP\n";
        "return from function"
    )]
    #[test_case(
        ParsedCmd::Flow(Flow::Call("Main.test".to_owned(), 0)),
        "//\n@ASM.Main.test$ret.1\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@LCL\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@ARG\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@SP\nD=M\n@5\nD=D-A\n@ARG\nM=D\n@SP\nD=M\n@LCL\nM=D\n@Main.test\n0;JMP\n(ASM.Main.test$ret.1)\n";
        "call function with no arguments"
    )]
    #[test_case(
        ParsedCmd::Marker(Marker::Label("test".to_owned())),
        "//\n(test)\n\0\0\0\0\0";
        "label"
    )]
    #[test_case(
        ParsedCmd::Marker(Marker::Function("test".to_owned(), 4)),
        "//\n(test)\nD=0\n@SP\nA=M\nM=D\n@SP\nM=M+1\nD=0\n@SP\nA=M\nM=D\n@SP\nM=M+1\nD=0\n@SP\nA=M\nM=D\n@SP\nM=M+1\nD=0\n@SP\nA=M\nM=D\n@SP\nM=M+1\n";
        "declare function"
    )]
    fn test_asm_generation(cmd: ParsedCmd, expected_asm: &str) {
        let mut buff = make_buff();
        let mut writer = make_writer(&mut buff);
        let cmd = make_command(cmd);
        writer.write(cmd).unwrap();
        assert_eq!(expected_asm, std::str::from_utf8(&buff).unwrap())
    }
}
