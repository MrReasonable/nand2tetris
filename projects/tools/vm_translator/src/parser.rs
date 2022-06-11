use std::{
    fmt::Display,
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
pub enum Flow {
    Label(String),
    Goto(Goto, String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Goto {
    Direct,
    Conditional,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParsedCmd {
    Arithmetic(Arithmetic),
    Push(PushSegment, HackMemSize),
    Pop(PopSegment, HackMemSize),
    Flow(Flow),
    Terminate,
    Noop,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum Segment {
    PushSegment(PushSegment),
    PopSegment(PopSegment),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum PushSegment {
    Argument,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp,
    Constant,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum PopSegment {
    Argument,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp,
}

impl From<PushSegment> for Segment {
    fn from(p: PushSegment) -> Self {
        Segment::PushSegment(p)
    }
}

impl From<PopSegment> for Segment {
    fn from(p: PopSegment) -> Self {
        Segment::PopSegment(p)
    }
}

impl Display for PushSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for PopSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

static STR_PUSH_SEGMENT: phf::Map<&str, PushSegment> = phf_map! {
    "argument" =>  PushSegment::Argument,
    "local" =>  PushSegment::Local,
    "static" =>  PushSegment::Static,
    "this" =>  PushSegment::This,
    "that" =>  PushSegment::That,
    "pointer" =>  PushSegment::Pointer,
    "temp" =>  PushSegment::Temp,
    "constant" => PushSegment::Constant,
};

static STR_POP_SEGMENT: phf::Map<&str, PopSegment> = phf_map! {
    "argument" =>  PopSegment::Argument,
    "local" =>  PopSegment::Local,
    "static" =>  PopSegment::Static,
    "this" =>  PopSegment::This,
    "that" =>  PopSegment::That,
    "pointer" =>  PopSegment::Pointer,
    "temp" =>  PopSegment::Temp
};

impl TryFrom<&str> for ParsedCmd {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = &value[..value.find("//").unwrap_or(value.len())];
        match value
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] => Ok(ParsedCmd::Noop),
            [arithmetic_cmd] => Ok(ParsedCmd::Arithmetic(
                STR_ARITHMETIC.get(arithmetic_cmd).map_or_else(
                    || Err(ParseError::UnknownCommandError(arithmetic_cmd.to_string())),
                    |cmd| Ok(*cmd),
                )?,
            )),
            ["label", label] => Ok(ParsedCmd::Flow(Flow::Label(label.to_string()))),
            ["if-goto", label] => Ok(ParsedCmd::Flow(Flow::Goto(
                Goto::Conditional,
                label.to_string(),
            ))),
            ["goto", label] => Ok(ParsedCmd::Flow(Flow::Goto(Goto::Direct, label.to_string()))),
            [op, segment, location] if *op == "push" || *op == "pop" => {
                let location = str::parse::<HackMemSize>(*location)?;
                if *op == "push" {
                    let segment = *STR_PUSH_SEGMENT.get(segment).map_or_else(
                        || Err(ParseError::UnknownSegmentError(segment.to_string())),
                        Ok,
                    )?;
                    Ok(ParsedCmd::Push(segment, location))
                } else {
                    let segment = *STR_POP_SEGMENT.get(segment).map_or_else(
                        || Err(ParseError::UnknownSegmentError(segment.to_string())),
                        Ok,
                    )?;
                    Ok(ParsedCmd::Pop(segment, location))
                }
            }
            tokens => Err(ParseError::UnknownCommandError(tokens.join(" "))),
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
    use test_case::test_case;

    fn make_cmd(cmd: &str) -> Command {
        let v = format!("{}", cmd).to_string();
        let c = io::Cursor::new(v);
        let r = BufReader::new(c);
        let mut parser = Parser::new(r);
        parser.next().transpose().unwrap().unwrap()
    }

    #[test]
    fn it_should_return_none_when_no_more_commands_available() {
        let c = io::Cursor::new(Vec::new());
        let r = BufReader::new(c);
        let mut parser = Parser::new(r);
        assert!(parser.next().is_none())
    }

    #[test_case(
        "add",
        Arithmetic::Add;
        "add"
    )]
    #[test_case(
        "sub",
        Arithmetic::Sub;
        "subtract"
    )]
    #[test_case(
        "neg",
        Arithmetic::Neg;
        "negation"
    )]
    #[test_case(
        "eq",
        Arithmetic::Eq;
        "equality"
    )]
    #[test_case(
        "gt",
        Arithmetic::Gt;
        "greater than"
    )]
    #[test_case(
        "lt",
        Arithmetic::Lt;
        "less than"
    )]
    #[test_case(
        "and",
        Arithmetic::And;
        "and"
    )]
    #[test_case(
        "or",
        Arithmetic::Or;
        "or"
    )]
    #[test_case(
        "not",
        Arithmetic::Not;
        "not"
    )]
    fn it_should_return_arithmetic_command_for_arithmetic_string(
        cmd_string: &str,
        cmd: Arithmetic,
    ) {
        assert_eq!(make_cmd(cmd_string).parsed(), &ParsedCmd::Arithmetic(cmd))
    }

    #[test_case(
        "label LOOP_START",
        Flow::Label("LOOP_START".to_string());
        "label"
    )]
    #[test_case(
        "if-goto LOOP_START",
        Flow::Goto(Goto::Conditional, "LOOP_START".to_string());
        "if-goto"
    )]
    fn it_should_return_flow_command_for_flow_string(flow_string: &str, flow_cmd: Flow) {
        assert_eq!(make_cmd(flow_string).parsed(), &ParsedCmd::Flow(flow_cmd));
    }

    #[test_case(
        "argument",
        PushSegment::Argument;
        "argument"
    )]
    #[test_case(
        "local",
        PushSegment::Local;
        "local"
    )]
    #[test_case(
        "static",
        PushSegment::Static;
        "static segment"
    )]
    #[test_case(
        "this",
        PushSegment::This;
        "this"
    )]
    #[test_case(
        "that",
        PushSegment::That;
        "that"
    )]
    #[test_case(
        "pointer",
        PushSegment::Pointer;
        "pointer"
    )]
    #[test_case(
        "temp",
        PushSegment::Temp;
        "temp"
    )]
    #[test_case(
        "constant",
        PushSegment::Constant;
        "constant segment"
    )]
    fn it_should_return_push_for_push_string(segment_str: &str, segment_cmd: PushSegment) {
        let v = format!("push {} 3", segment_str);
        assert_eq!(
            make_cmd(v.as_str()).parsed(),
            &ParsedCmd::Push(segment_cmd, 3)
        )
    }

    #[test_case(
        "argument",
        PopSegment::Argument;
        "argument"
    )]
    #[test_case(
        "local",
        PopSegment::Local;
        "local"
    )]
    #[test_case(
        "static",
        PopSegment::Static;
        "static segment"
    )]
    #[test_case(
        "this",
        PopSegment::This;
        "this"
    )]
    #[test_case(
        "that",
        PopSegment::That;
        "that"
    )]
    #[test_case(
        "pointer",
        PopSegment::Pointer;
        "pointer"
    )]
    #[test_case(
        "temp",
        PopSegment::Temp;
        "temp"
    )]
    fn it_should_return_pop_for_pop_string(segment_str: &str, segment_cmd: PopSegment) {
        let v = format!("pop {} 3", segment_str);
        assert_eq!(
            make_cmd(v.as_str()).parsed(),
            &ParsedCmd::Pop(segment_cmd, 3)
        )
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
    fn it_should_return_error_when_unknown_segment_supplied() {
        for segment in vec!["nosegment", "constant"] {
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
