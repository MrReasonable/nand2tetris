#[derive(Debug, PartialEq)]
pub enum Symbol<'a> {
    Label(&'a str),
    AInstruction(&'a str),
    CInstruction(CInstruction<'a>)
}

#[derive(Debug, PartialEq)]
pub enum TokenError {
    UnclosedLabelError,
    EmptyAInstructionError,
    InvalidSymbolFirstCharError(String),
    InvalidSymbolCharError(String),
    UnexpectedCharacterError(String),
    MissingCmpError,
}

#[derive(Debug, PartialEq)]
pub struct CInstruction<'a> {
    dest: Option<&'a str>,
    comp: &'a str,
    jump: Option<&'a str>
}

impl<'a> CInstruction<'a> {
    fn new(dest: Option<&'a str>, comp: &'a str, jump: Option<&'a str>) -> CInstruction<'a> {
        CInstruction {
            dest,
            comp,
            jump
        }
    }

    pub fn dest(&self) -> Option<&str> {
        self.dest
    }

    pub fn comp(&self) -> &str {
        self.comp
    }

    pub fn jump(&self) -> Option<&str> {
        self.jump
    }
}

use Symbol::*;
use TokenError::*;

pub fn tokenize(line: &str) -> Result<Option<Symbol>, TokenError> {
    tokenize_with_index(line, 0)
}

fn tokenize_with_index(line: &str, mut idx: usize) -> Result<Option<Symbol>, TokenError> {
    let trimmed_line = strip_comments(line).trim();
    if trimmed_line.is_empty() {
        return Ok(None)
    }

    for c in trimmed_line.chars() {
        idx += 1;
        match c {
            ' ' => continue,
            '(' => {
                return extract_label(trimmed_line, idx)
            },
            '@' => {
                return extract_a_instruction(trimmed_line, idx)
            }
            _ => return extract_c_instruction(trimmed_line),
        }
    }

    Ok(None)
}

fn extract_label(line: &str, start_idx: usize) -> Result<Option<Symbol>, TokenError> {
    let mut idx = start_idx;
    if !is_valid_symbol_first_char(line.chars().nth(start_idx).unwrap()) {
        return Err(InvalidSymbolFirstCharError(format!("'{}' at position {} is not a valid start character for a Symbol.  Symbol may only start with [a-zA-Z.$:_]", line.chars().next().unwrap(), idx)))
    }
    let length = line.len();
    for c in line.chars().skip(start_idx) {
        idx +=1; 
        println!{"idx: {}, start_idx: {}, c: {}", idx, start_idx, c}
        match c {
            ')' => {
                break;
            },
            _  if !is_valid_symbol(c) => {
                    return Err(InvalidSymbolCharError(
                        format!("'{}' at position {} is not a valid character for a Symbol.  Symbol may only contain [a-zA-Z0-9.$:_]", c, idx-1)
                    ))
                    },
            _ if length <= idx => return Err(UnclosedLabelError),
            _ => continue
        }
    }

    if length > idx {
        Err(UnexpectedCharacterError(format!("'{}' on string '{}' at position {}", &line[idx..idx+1], &line, idx)))
    } else {
        Ok(Some(Label(&line[start_idx..idx-1])))
    }
}

fn extract_a_instruction(line: &str, start_idx: usize) -> Result<Option<Symbol>, TokenError> {
    let mut idx = start_idx;
    if line.len() <= idx {
        Err(EmptyAInstructionError)
    } else if !is_valid_symbol_first_char(line.chars().nth(start_idx).unwrap()) {
        Err(InvalidSymbolFirstCharError(
            format!("'{}' at position {} is not a valid start character for a Symbol.  Symbol may only start with [a-zA-Z.$:_]", 
            line.chars().next().unwrap(), 
            idx)
        ))
    } else {
        for c in line.chars().skip(start_idx) {
            if !is_valid_symbol(c) {
                return Err(InvalidSymbolCharError(
                    format!("'{}' at position {} is not a valid character for a Symbol.  Symbol may only contain [a-zA-Z0-9.$:_]", 
                    c, 
                    idx-1)
                ))
            }
            idx += 1;
        }
        Ok(Some(AInstruction(&line[start_idx..])))
    }
}

fn extract_c_instruction(line: &str) -> Result<Option<Symbol>, TokenError> {
    let (dest, cmp_string) = match line.find('=') {
        Some(idx) => (Some(&line[..idx]), &line[idx+1..]),
        None => (None, line)
    };
    let (cmp, jmp) = match cmp_string.find(';') {
        Some(idx) => (Some(&cmp_string[..idx]), Some(&cmp_string[idx+1..])),
        None => (Some(cmp_string), None)
    };
    if cmp == None {
        Err(MissingCmpError)
    } else {
        Ok(Some(CInstruction(CInstruction::new(dest, cmp.unwrap_or_default(), jmp))))
    }
}

fn is_valid_symbol_first_char(c: char) -> bool {
    is_valid_symbol(c) && !c.is_digit(10)
}

fn is_valid_symbol(c: char) -> bool {
    c.is_ascii() && (c.is_alphabetic() || c.is_digit(10) || 
        c == '_' || c == '.' || c == '$' || c == ':'
    )
}

fn strip_comments(line: &str) -> &str {
    match line.find("//") {
        None => line,
        Some(size) => &line[0..size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_ignores_comments() {
       assert_eq!(strip_comments("//test"), "");
       assert_eq!(strip_comments("//test    "), "");
       assert_eq!(strip_comments("    //test    "), "    ");
       assert_eq!(strip_comments("before comment//test"), "before comment");
       assert_eq!(strip_comments("before comment    //test"), "before comment    ");
       assert_eq!(strip_comments("    before comment    //test"), "    before comment    ");
    }

    #[test]
    fn it_ignores_trailing_whitespace() {
        assert_eq!(tokenize("//test    "), Ok(None));
        assert_eq!(tokenize("    //test    "), Ok(None));
    }

    #[test]
    fn it_treats_empty_string_as_none() {
        assert_eq!(tokenize("   "), Ok(None));
        assert_eq!(tokenize(""), Ok(None));
    }

    #[test]
    fn it_extracts_label() {
        assert_eq!(tokenize("(test)"), Ok(Some(Label("test"))));
        assert_eq!(tokenize("(test1)"), Ok(Some(Label("test1"))));
        assert_eq!(tokenize("(test$)"), Ok(Some(Label("test$"))));
        assert_eq!(tokenize("(test_)"), Ok(Some(Label("test_"))));
        assert_eq!(tokenize("(test.)"), Ok(Some(Label("test."))));
        assert_eq!(tokenize("(test:)"), Ok(Some(Label("test:"))));
        assert_eq!(tokenize("(test)//with comment"), Ok(Some(Label("test"))));
        assert_eq!(tokenize("(test)   //with trailing whitespace and comment"), Ok(Some(Label("test"))));
        assert_eq!(tokenize("    (test)//with leading whitespace and comment"), Ok(Some(Label("test"))));
        assert_eq!(tokenize("    (test)    //with leading and trailing whitespace and comment"), Ok(Some(Label("test"))));
    }

    #[test]
    fn it_detects_unexpected_character_after_label_close() {
        assert!(matches!(tokenize("(test)1"), Err(TokenError::UnexpectedCharacterError(_))))
    }

    #[test]
    fn it_detects_missing_closing_character_for_label() {
        assert_eq!(tokenize("(test"), Err(TokenError::UnclosedLabelError))
    }

    #[test]
    fn it_detects_invalid_characters_in_label() {
        assert!(matches!(tokenize("(1test)"), Err(TokenError::InvalidSymbolFirstCharError(_))));
        assert!(matches!(tokenize("(t\"est)"), Err(TokenError::InvalidSymbolCharError(_))));
    }


    #[test]
    fn it_extracts_a_instr() {
        assert_eq!(tokenize("@test"), Ok(Some(AInstruction("test"))));
        assert_eq!(tokenize("@test1"), Ok(Some(AInstruction("test1"))));
        assert_eq!(tokenize("@test$"), Ok(Some(AInstruction("test$"))));
        assert_eq!(tokenize("@test_"), Ok(Some(AInstruction("test_"))));
        assert_eq!(tokenize("@test."), Ok(Some(AInstruction("test."))));
        assert_eq!(tokenize("@test:"), Ok(Some(AInstruction("test:"))));
        assert_eq!(tokenize("@test//with comment"), Ok(Some(AInstruction("test"))));
        assert_eq!(tokenize("@test   //with trailing whitespace and comment"), Ok(Some(AInstruction("test"))));
        assert_eq!(tokenize("    @test//with leading whitespace and comment"), Ok(Some(AInstruction("test"))));
        assert_eq!(tokenize("    @test    //with leading and trailing whitespace and comment"), Ok(Some(AInstruction("test"))));
    }

    #[test]
    fn it_detects_invalid_characters_in_a_instr() {
        assert!(matches!(tokenize("@1test"), Err(TokenError::InvalidSymbolFirstCharError(_))));
        assert!(matches!(tokenize("@t\"est"), Err(TokenError::InvalidSymbolCharError(_))));
    }

    #[test]
    fn it_extracts_single_compute_command() {
        assert_eq!(tokenize("D"), Ok(Some(CInstruction(CInstruction::new(None, "D", None)))));
        assert_eq!(tokenize(" A "), Ok(Some(CInstruction(CInstruction::new(None, "A", None)))));
        assert_eq!(tokenize(" A //some comment"), Ok(Some(CInstruction(CInstruction::new(None, "A", None)))));
    }

    #[test]
    fn it_extracts_compute_command_with_destination() {
        assert_eq!(tokenize("D=0"), Ok(Some(CInstruction(CInstruction::new(Some("D"), "0", None)))));
        assert_eq!(tokenize("D=M"), Ok(Some(CInstruction(CInstruction::new(Some("D"), "M", None)))));
    }

    #[test]
    fn it_extracts_compute_command_with_jump() {
        assert_eq!(tokenize("0;JMP"), Ok(Some(CInstruction(CInstruction::new(None, "0", Some("JMP"))))));
        assert_eq!(tokenize("D;JMP"), Ok(Some(CInstruction(CInstruction::new(None, "D", Some("JMP"))))));
    }

    #[test]
    fn it_extracts_compute_command_with_destination_and_jump() {
        assert_eq!(tokenize("D=0;JMP"), Ok(Some(CInstruction(CInstruction::new(Some("D"), "0", Some("JMP"))))));
        assert_eq!(tokenize("D=A+1;JLE"), Ok(Some(CInstruction(CInstruction::new(Some("D"), "A+1", Some("JLE"))))));
        assert_eq!(tokenize("AMD=D+1;JEQ"), Ok(Some(CInstruction(CInstruction::new(Some("AMD"), "D+1", Some("JEQ"))))));
    }
}
