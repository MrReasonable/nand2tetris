use std::{
    io::{self, BufRead, BufReader, Lines, Read},
    iter::Peekable,
    num::ParseIntError,
};

use phf::phf_map;

pub type HackMemSize = u16;

#[derive(Debug, PartialEq)]
pub struct Command {
    original: String,
    parsed: ParsedCmd,
}

impl Command {
    pub fn new(original: String, parsed_command: ParsedCmd) -> Self {
        Self {
            original,
            parsed: parsed_command,
        }
    }
    pub fn original(&self) -> &String {
        &self.original
    }

    pub fn parsed(&self) -> &ParsedCmd {
        &self.parsed
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("file error: {0}")]
    IoError(#[from] io::Error),
    #[error("unknown command: {0}")]
    UnknownCommandError(String),
    #[error("unknown segment: {0}")]
    UnknownSegmentError(String),
    #[error("invalid memory location: {0}")]
    InvalidMemoryLocation(#[from] ParseIntError),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Arithmetic {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParsedCmd {
    Arithmetic(Arithmetic),
    Constant(HackMemSize),
    Push(Stack, HackMemSize),
    Pop(Stack, HackMemSize),
    Label(String),
    Goto(String),
    If(String),
    Function(String, HackMemSize),
    Call(String, HackMemSize),
    Return,
    Noop,
}

#[derive(Debug, PartialEq)]
enum StackType {
    Stack(Stack),
    Constant,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum Stack {
    Argument,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp,
}

#[derive(Debug)]
pub struct Parser<R: Read> {
    in_stream: Peekable<Lines<BufReader<R>>>,
}

static STR_ARITHMETIC: phf::Map<&str, Arithmetic> = phf_map! {
    "add" =>  Arithmetic::Add,
    "sub" =>  Arithmetic::Sub,
    "neg" =>  Arithmetic::Neg,
    "eq" =>  Arithmetic::Eq,
    "gt" =>  Arithmetic::Gt,
    "lt" =>  Arithmetic::Lt,
    "and" =>  Arithmetic::And,
    "or" =>  Arithmetic::Or,
    "not" =>  Arithmetic::Not,
};

static STR_STACK: phf::Map<&str, Stack> = phf_map! {
    "argument" =>  Stack::Argument,
    "local" =>  Stack::Local,
    "static" =>  Stack::Static,
    "this" =>  Stack::This,
    "that" =>  Stack::That,
    "pointer" =>  Stack::Pointer,
    "temp" =>  Stack::Temp,
};

impl TryFrom<&str> for ParsedCmd {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = &value[..value.find("//").unwrap_or(value.len())];
        match value.split_ascii_whitespace().collect::<Vec<&str>>() {
            ref tokens if tokens.is_empty() => Ok(ParsedCmd::Noop),
            ref tokens if tokens.len() == 1 => {
                if tokens[0] == "return" {
                    Ok(ParsedCmd::Return)
                } else {
                    Ok(ParsedCmd::Arithmetic(
                        STR_ARITHMETIC.get(tokens[0]).map_or_else(
                            || Err(ParseError::UnknownCommandError(tokens[0].to_owned())),
                            |cmd| Ok(*cmd),
                        )?,
                    ))
                }
            }
            ref tokens if tokens.len() == 3 => {
                let stack = if tokens[1] != "constant" {
                    StackType::Stack(*STR_STACK.get(tokens[1]).map_or_else(
                        || Err(ParseError::UnknownSegmentError(tokens[1].to_owned())),
                        Ok,
                    )?)
                } else {
                    StackType::Constant
                };
                let location = str::parse::<HackMemSize>(tokens[2])?;
                if tokens[0] == "push" {
                    match stack {
                        StackType::Stack(stack) => Ok(ParsedCmd::Push(stack, location)),
                        StackType::Constant => Ok(ParsedCmd::Constant(location)),
                    }
                } else {
                    match stack {
                        StackType::Stack(stack) => Ok(ParsedCmd::Pop(stack, location)),
                        StackType::Constant => {
                            Err(ParseError::UnknownSegmentError("constant".to_owned()))?
                        }
                    }
                }
            }
            ref tokens => Err(ParseError::UnknownCommandError(tokens.join(" "))),
        }
    }
}

impl<R: Read> Parser<R> {
    pub fn new(in_stream: BufReader<R>) -> Self {
        Parser {
            in_stream: in_stream.lines().peekable(),
        }
    }
}

impl<R: Read> Iterator for Parser<R> {
    type Item = Result<Command, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.in_stream.next() {
            match line {
                Ok(line) => match line.as_str().try_into() {
                    Ok(parsed_cmd) => Some(Ok(Command::new(line.clone(), parsed_cmd))),
                    Err(err) => Some(Err(err)),
                },
                Err(line) => Some(Err(line.into())),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn it_should_return_none_when_no_more_commands_available() {
        let c = io::Cursor::new(Vec::new());
        let r = BufReader::new(c);
        let mut parser = Parser::new(r);
        assert!(parser.next().is_none())
    }

    #[test]
    fn it_should_return_arithmetic_command_for_arithmetic_string() {
        let arithmetic_strings = vec!["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];
        let arithmetic_cmd = vec![
            Arithmetic::Add,
            Arithmetic::Sub,
            Arithmetic::Neg,
            Arithmetic::Eq,
            Arithmetic::Gt,
            Arithmetic::Lt,
            Arithmetic::And,
            Arithmetic::Or,
            Arithmetic::Not,
        ];
        for (i, cmd) in arithmetic_strings.into_iter().enumerate() {
            let v = format!("{}", cmd).to_string();
            let c = io::Cursor::new(v);
            let r = BufReader::new(c);
            let mut parser = Parser::new(r);
            let cmd = parser.next().transpose().unwrap().unwrap();
            assert_eq!(cmd.parsed(), &ParsedCmd::Arithmetic(arithmetic_cmd[i]))
        }
    }

    #[test]
    fn it_should_return_push_for_push_string() {
        let stack_strings = vec![
            "argument", "local", "static", "this", "that", "pointer", "temp",
        ];
        let stacks = vec![
            Stack::Argument,
            Stack::Local,
            Stack::Static,
            Stack::This,
            Stack::That,
            Stack::Pointer,
            Stack::Temp,
        ];
        for (k, v) in stack_strings.into_iter().enumerate() {
            let v = format!("push {} 3", v).to_string();
            let c = io::Cursor::new(v);
            let r = BufReader::new(c);
            let mut parser = Parser::new(r);
            let cmd = parser.next().transpose().unwrap().unwrap();
            assert_eq!(cmd.parsed(), &ParsedCmd::Push(stacks[k], 3))
        }
    }

    #[test]
    fn it_should_return_constant_for_push_to_constant() {
        let v = "push constant 3".to_string();
        let c = io::Cursor::new(v);
        let r = BufReader::new(c);
        let mut parser = Parser::new(r);
        let cmd = parser.next().transpose().unwrap().unwrap();
        assert_eq!(cmd.parsed(), &ParsedCmd::Constant(3))
    }

    #[test]
    fn it_should_return_pop_for_pop_string() {
        let stack_strings = vec![
            "argument", "local", "static", "this", "that", "pointer", "temp",
        ];
        let stacks = vec![
            Stack::Argument,
            Stack::Local,
            Stack::Static,
            Stack::This,
            Stack::That,
            Stack::Pointer,
            Stack::Temp,
        ];
        for (i, stack) in stack_strings.into_iter().enumerate() {
            let v = format!("pop {} 3", stack).to_string();
            let c = io::Cursor::new(v);
            let r = BufReader::new(c);
            let mut parser = Parser::new(r);
            let cmd = parser.next().transpose().unwrap().unwrap();
            assert_eq!(cmd.parsed(), &ParsedCmd::Pop(stacks[i], 3))
        }
    }

    #[test]
    fn it_should_return_error_when_unknown_single_command_supplied() {
        let v = "wrong".to_string();
        let c = io::Cursor::new(v);
        let r = BufReader::new(c);
        let mut parser = Parser::new(r);
        assert_matches!(
            parser.next().transpose(),
            Err(ParseError::UnknownCommandError(s)) => {
                assert_eq!(s, "wrong".to_owned())
            }
        );
    }

    #[test]
    fn it_should_return_error_when_unknown_stack_supplied() {
        for segment in vec!["nostack", "constant"] {
            let v = format!("pop {} 3", segment);
            let c = io::Cursor::new(v);
            let r = BufReader::new(c);
            let mut parser = Parser::new(r);
            assert_matches!(
                parser.next().transpose(),
                Err(ParseError::UnknownSegmentError(s)) => {
                    assert_eq!(s, segment)
                }
            );
        }
    }
}
