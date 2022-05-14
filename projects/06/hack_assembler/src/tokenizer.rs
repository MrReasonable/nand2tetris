use std::fmt::{Display};

use thiserror::Error;

use crate::{instructions::{CInstruction, AInstruction}, symbol_table::HackMemSize};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Label(String),
    AInstruction(AInstruction),
    CInstruction(CInstruction)
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Token::Label(l) => write!(f, "label: {}", l),
            Token::AInstruction(a) => write!(f, "ainstr: {}", a),
            Token::CInstruction(c) => write!(f, "cinstr: {}", c),
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum TokenError {
    #[error("unclosed label")]
    UnclosedLabelError,
    #[error("Attempt to define alias without providing a name")]
    EmptyAInstructionError,
    #[error("invalid symbol for first character: {0}")]
    InvalidSymbolFirstChar(String),
    #[error("invalid symbol error: {0}")]
    InvalidSymbolChar(String),
    #[error("unexpected character error: {0}")]
    UnexpectedCharacter(String),
    #[error("missing computation instruction")]
    MissingCmpInstruction,
}

pub fn tokenize(line: &str) -> Result<Option<Token>, TokenError> {
    tokenize_with_index(line, 0)
}

fn tokenize_with_index(line: &str, mut idx: usize) -> Result<Option<Token>, TokenError> {
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

fn extract_label(line: &str, start_idx: usize) -> Result<Option<Token>, TokenError> {
    let mut idx = start_idx;
    if !is_valid_symbol_first_char(line.chars().nth(start_idx).unwrap()) {
        return Err(TokenError::InvalidSymbolFirstChar(format!("'{}' at position {} is not a valid start character for a Symbol.  Symbol may only start with [a-zA-Z.$:_]", line.chars().next().unwrap(), idx)))
    }
    let length = line.len();
    for c in line.chars().skip(start_idx) {
        idx +=1; 
        match c {
            ')' => {
                break;
            },
            _  if !is_valid_symbol(c) => {
                    return Err(TokenError::InvalidSymbolChar(
                        format!("'{}' at position {} is not a valid character for a Symbol.  Symbol may only contain [a-zA-Z0-9.$:_]", c, idx-1)
                    ))
                    },
            _ if length <= idx => return Err(TokenError::UnclosedLabelError),
            _ => continue
        }
    }

    if length > idx {
        Err(TokenError::UnexpectedCharacter(format!("'{}' on string '{}' at position {}", &line[idx..idx+1], &line, idx)))
    } else {
        Ok(Some(Token::Label(line[start_idx..idx-1].to_string())))
    }
}

fn extract_a_instruction(line: &str, start_idx: usize) -> Result<Option<Token>, TokenError> {
    let mut idx = start_idx;
    if let Ok(addr) = line[start_idx..].parse::<HackMemSize>() {
        return Ok(Some(Token::AInstruction(AInstruction::RawAddr(addr))));
    }

    if line.len() <= idx {
        Err(TokenError::EmptyAInstructionError)
    } else if !is_valid_symbol_first_char(line.chars().nth(start_idx).unwrap()) {
        Err(TokenError::InvalidSymbolFirstChar(
            format!("'{}' at position {} is not a valid start character for a Symbol.  Symbol may only start with [a-zA-Z.$:_]", 
            line.chars().next().unwrap(), 
            idx)
        ))
    } else {
        for c in line.chars().skip(start_idx) {
            if !is_valid_symbol(c) {
                return Err(TokenError::InvalidSymbolChar(
                    format!("'{}' at position {} is not a valid character for a Symbol.  Symbol may only contain [a-zA-Z0-9.$:_]", 
                    c, 
                    idx-1)
                ))
            }
            idx += 1;
        }
        let ainst = AInstruction::Alias(line[start_idx..].to_string());
        Ok(Some(Token::AInstruction(ainst)))
    }
}

fn extract_c_instruction(line: &str) -> Result<Option<Token>, TokenError> {
    let (dest, cmp_string) = match line.find('=') {
        Some(idx) => (Some(line[..idx].to_string()), &line[idx+1..]),
        None => (None, line)
    };
    let (cmp, jmp) = match cmp_string.find(';') {
        Some(idx) => (Some(cmp_string[..idx].to_string()), Some(cmp_string[idx+1..].to_string())),
        None => (Some(cmp_string.to_string()), None)
    };
    if cmp == None {
        Err(TokenError::MissingCmpInstruction)
    } else {
        Ok(Some(Token::CInstruction(CInstruction::new(dest, cmp.unwrap_or_default(), jmp))))
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
        assert_eq!(tokenize("(test)"), Ok(Some(Token::Label("test".to_string()))));
        assert_eq!(tokenize("(test1)"), Ok(Some(Token::Label("test1".to_string()))));
        assert_eq!(tokenize("(test$)"), Ok(Some(Token::Label("test$".to_string()))));
        assert_eq!(tokenize("(test_)"), Ok(Some(Token::Label("test_".to_string()))));
        assert_eq!(tokenize("(test.)"), Ok(Some(Token::Label("test.".to_string()))));
        assert_eq!(tokenize("(test:)"), Ok(Some(Token::Label("test:".to_string()))));
        assert_eq!(tokenize("(test)//with comment"), Ok(Some(Token::Label("test".to_string()))));
        assert_eq!(tokenize("(test)   //with trailing whitespace and comment"), Ok(Some(Token::Label("test".to_string()))));
        assert_eq!(tokenize("    (test)//with leading whitespace and comment"), Ok(Some(Token::Label("test".to_string()))));
        assert_eq!(tokenize("    (test)    //with leading and trailing whitespace and comment"), Ok(Some(Token::Label("test".to_string()))));
    }

    #[test]
    fn it_detects_unexpected_character_after_label_close() {
        assert!(matches!(tokenize("(test)1"), Err(TokenError::UnexpectedCharacter(_))))
    }

    #[test]
    fn it_detects_missing_closing_character_for_label() {
        assert_eq!(tokenize("(test"), Err(TokenError::UnclosedLabelError))
    }

    #[test]
    fn it_detects_invalid_characters_in_label() {
        assert!(matches!(tokenize("(1test)"), Err(TokenError::InvalidSymbolFirstChar(_))));
        assert!(matches!(tokenize("(t\"est)"), Err(TokenError::InvalidSymbolChar(_))));
    }


    #[test]
    fn it_extracts_a_instr() {
        assert_eq!(tokenize("@test"), Ok(Some(Token::AInstruction(AInstruction::Alias("test".to_string())))));
        assert_eq!(tokenize("@test1"), Ok(Some(Token::AInstruction(AInstruction::Alias("test1".to_string())))));
        assert_eq!(tokenize("@test$"), Ok(Some(Token::AInstruction(AInstruction::Alias("test$".to_string())))));
        assert_eq!(tokenize("@test_"), Ok(Some(Token::AInstruction(AInstruction::Alias("test_".to_string())))));
        assert_eq!(tokenize("@test."), Ok(Some(Token::AInstruction(AInstruction::Alias("test.".to_string())))));
        assert_eq!(tokenize("@test:"), Ok(Some(Token::AInstruction(AInstruction::Alias("test:".to_string())))));
        assert_eq!(tokenize("@test//with comment"), Ok(Some(Token::AInstruction(AInstruction::Alias("test".to_string())))));
        assert_eq!(tokenize("@test   //with trailing whitespace and comment"), Ok(Some(Token::AInstruction(AInstruction::Alias("test".to_string())))));
        assert_eq!(tokenize("    @test//with leading whitespace and comment"), Ok(Some(Token::AInstruction(AInstruction::Alias("test".to_string())))));
        assert_eq!(tokenize("    @test    //with leading and trailing whitespace and comment"), Ok(Some(Token::AInstruction(AInstruction::Alias("test".to_string())))));
        assert_eq!(tokenize("@123"), Ok(Some(Token::AInstruction(AInstruction::RawAddr(123)))));
    }

    #[test]
    fn it_detects_invalid_characters_in_a_instr() {
        assert!(matches!(tokenize("@1test"), Err(TokenError::InvalidSymbolFirstChar(_))));
        assert!(matches!(tokenize("@t\"est"), Err(TokenError::InvalidSymbolChar(_))));
    }

    #[test]
    fn it_extracts_single_compute_command() {
        assert_eq!(tokenize("D"), Ok(Some(Token::CInstruction(CInstruction::new(None, "D".to_string(), None)))));
        assert_eq!(tokenize(" A "), Ok(Some(Token::CInstruction(CInstruction::new(None, "A".to_string(), None)))));
        assert_eq!(tokenize(" A //some comment"), Ok(Some(Token::CInstruction(CInstruction::new(None, "A".to_string(), None)))));
    }

    #[test]
    fn it_extracts_compute_command_with_destination() {
        assert_eq!(tokenize("D=0"), Ok(Some(Token::CInstruction(CInstruction::new(Some("D".to_string()), "0".to_string(), None)))));
        assert_eq!(tokenize("D=M"), Ok(Some(Token::CInstruction(CInstruction::new(Some("D".to_string()), "M".to_string(), None)))));
    }

    #[test]
    fn it_extracts_compute_command_with_jump() {
        assert_eq!(tokenize("0;JMP"), Ok(Some(Token::CInstruction(CInstruction::new(None, "0".to_string(), Some("JMP".to_string()))))));
        assert_eq!(tokenize("D;JMP"), Ok(Some(Token::CInstruction(CInstruction::new(None, "D".to_string(), Some("JMP".to_string()))))));
    }

    #[test]
    fn it_extracts_compute_command_with_destination_and_jump() {
        assert_eq!(tokenize("D=0;JMP"), Ok(Some(Token::CInstruction(CInstruction::new(Some("D".to_string()), "0".to_string(), Some("JMP".to_string()))))));
        assert_eq!(tokenize("D=A+1;JLE"), Ok(Some(Token::CInstruction(CInstruction::new(Some("D".to_string()), "A+1".to_string(), Some("JLE".to_string()))))));
        assert_eq!(tokenize("AMD=D+1;JEQ"), Ok(Some(Token::CInstruction(CInstruction::new(Some("AMD".to_string()), "D+1".to_string(), Some("JEQ".to_string()))))));
    }
}
