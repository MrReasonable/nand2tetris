use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
};

use crate::parser::{Arithmetic, Command, ParsedCmd, Stack};

#[derive(thiserror::Error, Debug)]
pub enum CodeWriterError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
}

pub struct CodeWriter<W: Write> {
    out_stream: BufWriter<W>,
    namespace: String,
}

lazy_static! {
    static ref MEM_MAP: HashMap<Stack, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Stack::Local, "LCL");
        m.insert(Stack::Argument, "ARG");
        m.insert(Stack::This, "THIS");
        m.insert(Stack::That, "THAT");
        m
    };
}

impl<W: Write> CodeWriter<W> {
    pub fn new(out_stream: BufWriter<W>) -> Self {
        Self {
            out_stream,
            namespace: "".to_owned(),
        }
    }

    pub fn set_namespace(&mut self, namespace: &str) {
        self.namespace = namespace.to_owned();
    }

    pub fn comment(&mut self, comment: &str) -> Result<(), CodeWriterError> {
        Ok(writeln!(self.out_stream, "//{}", comment)?)
    }

    pub fn write(&mut self, cmd: Command) -> Result<(), CodeWriterError> {
        self.comment(cmd.original())?;
        if let Some(asm) = self.cmd_to_asm(cmd.parsed().clone()) {
            for line in asm {
                writeln!(self.out_stream, "{}", line)?;
            }
        };
        Ok(())
    }

    fn cmd_to_asm(&self, cmd: ParsedCmd) -> Option<Vec<String>> {
        match cmd {
            ParsedCmd::Arithmetic(arr) => Some(self.arithmetic_to_asm(arr)),
            ParsedCmd::Push(stack, value) => Some(self.push_to_asm(stack, value)),
            ParsedCmd::Constant(value) => Some(self.push_to_stack(value)),
            ParsedCmd::Pop(stack, value) => todo!(),
            ParsedCmd::Label(_) => todo!(),
            ParsedCmd::Goto(_) => todo!(),
            ParsedCmd::If(_) => todo!(),
            ParsedCmd::Function(_, _) => todo!(),
            ParsedCmd::Call(_, _) => todo!(),
            ParsedCmd::Return => todo!(),
            ParsedCmd::Noop => None,
        }
    }

    fn push_to_asm(&self, stack: Stack, value: u16) -> Vec<String> {
        match stack {
            Stack::Argument => todo!(),
            Stack::Local => todo!(),
            Stack::Static => todo!(),
            Stack::This => todo!(),
            Stack::That => todo!(),
            Stack::Pointer => todo!(),
            Stack::Temp => todo!(),
        }
    }

    fn push_to_stack(&self, value: u16) -> Vec<String> {
        vec![
            format!("@{}", value),
            "D=A".to_owned(),
            "@SP".to_owned(),
            "A=M".to_owned(),
            "M=D".to_owned(),
            "@SP".to_owned(),
            "M=M+1".to_owned(),
        ]
    }

    fn pop_to_asm(&self, stack: Stack, value: u16) -> Vec<String> {
        match stack {
            Stack::Argument => todo!(),
            Stack::Local => todo!(),
            Stack::Static => todo!(),
            Stack::This => todo!(),
            Stack::That => todo!(),
            Stack::Pointer => todo!(),
            Stack::Temp => todo!(),
        }
    }

    fn arithmetic_to_asm(&self, arr: Arithmetic) -> Vec<String> {
        match arr {
            Arithmetic::Add => self.add_to_asm(),
            Arithmetic::Sub => todo!(),
            Arithmetic::Neg => todo!(),
            Arithmetic::Eq => todo!(),
            Arithmetic::Gt => todo!(),
            Arithmetic::Lt => todo!(),
            Arithmetic::And => todo!(),
            Arithmetic::Or => todo!(),
            Arithmetic::Not => todo!(),
        }
    }

    fn add_to_asm(&self) -> Vec<String> {
        vec![
            "@SP".to_owned(),
            "A=M".to_owned(),
            "D=M".to_owned(),
            "@SP".to_owned(),
            "M=M-1".to_owned(),
            "A=M".to_owned(),
            "M=D+M".to_owned(),
            "@SP".to_owned(),
            "M=M+1".to_owned(),
        ]
    }
}
