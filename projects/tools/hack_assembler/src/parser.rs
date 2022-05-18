use std::io::{BufReader, Read};

use crate::{
    instructions::{AInstruction, CInstruction},
    symbol_table::{HackRomSize, SymbolTable, SymbolTableError, START_CMP_INSTR},
    tokenizer::{tokenize, Token, TokenError},
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("tokenize error: {0}")]
    TokenError(#[from] TokenError),
    #[error("symbol table error: {0}")]
    SymbolTableError(#[from] SymbolTableError),
    #[error("non-compilable token: {0}")]
    NonCompilableToken(Token),
    #[error("address not found for alias: {0}")]
    AliasNotFound(String),
}

struct CInstWithSymbols<'a>(&'a CInstruction, &'a SymbolTable);

impl From<CInstWithSymbols<'_>> for u16 {
    fn from(cinst_with_symbols: CInstWithSymbols<'_>) -> Self {
        let (cinstr, symbols) = (cinst_with_symbols.0, cinst_with_symbols.1);
        let empty_str = &"".to_owned();
        let comp = cinstr.comp();
        let dest = cinstr.dest().unwrap_or(empty_str).as_str();
        let jump = cinstr.jump().unwrap_or(empty_str).as_str();
        let comp = symbols.get_comp_instr(comp).unwrap_or_default();
        let dest = symbols.get_dest_instr(dest).unwrap_or_default();
        let jump = symbols.get_jmp_instr(jump).unwrap_or_default();
        START_CMP_INSTR | comp | dest | jump
    }
}

pub fn parse<R>(source: &mut BufReader<R>) -> Result<Vec<String>, ParseError>
where
    R: Read,
{
    let mut code = String::new();
    source.read_to_string(&mut code)?;
    let (symbols, tokens) = first_pass(&code)?;
    let bin = convert_to_bin(symbols, tokens)?;
    let ret: Vec<String> = bin.iter().map(|b| bin_string(*b)).collect();
    Ok(ret)
}

fn first_pass(code: &str) -> Result<(SymbolTable, Vec<Token>), ParseError> {
    let mut symbols = SymbolTable::new();
    let mut tokens = Vec::new();

    for line in code.lines() {
        if let Some(token) = tokenize(line)? {
            match token {
                Token::Label(ref label) => {
                    symbols.add_label(label.clone(), (tokens.len()) as HackRomSize)?;
                }
                Token::CInstruction(_) | Token::AInstruction(_) => tokens.push(token),
            }
        }
    }
    Ok((symbols, tokens))
}

fn convert_to_bin(mut symbols: SymbolTable, tokens: Vec<Token>) -> Result<Vec<u16>, ParseError> {
    tokens
        .iter()
        .map(|token| match token {
            Token::AInstruction(a) => match a {
                AInstruction::RawAddr(addr) => Ok(*addr),
                AInstruction::Alias(alias) => {
                    if let Some(addr) = symbols.get_addr(alias) {
                        Ok(addr)
                    } else if let Some(addr) = symbols.get_line_no(alias) {
                        Ok(addr)
                    } else {
                        symbols
                            .add_alias(alias.clone())
                            .map_err(ParseError::SymbolTableError)
                    }
                }
            },
            Token::CInstruction(cinstr) => Ok(CInstWithSymbols(cinstr, &symbols).into()),
            token => Err(ParseError::NonCompilableToken(token.clone())),
        })
        .collect()
}

fn bin_string(mut val: u16) -> String {
    let mut ret_string = "".to_owned();
    for _ in 0..16 {
        let mut new_string = if 1_u16 & val == 0 {
            "0".to_owned()
        } else {
            "1".to_owned()
        };
        new_string.push_str(&ret_string);
        ret_string = new_string;
        val >>= 1;
    }
    ret_string
}

#[cfg(test)]
mod test {
    use std::{fs::File, path::Path};

    use super::*;

    fn setup(p: &Path) -> BufReader<File> {
        let reader = File::open(p).unwrap();
        BufReader::new(reader)
    }

    #[test]
    fn it_generates_expected_binary_code_for_input() {
        let mut reader = setup(Path::new("./test_files/Max.asm"));
        let parsed = parse(&mut reader).unwrap();
        let mut reader = setup(Path::new("./test_files/test-cmp.hack"));
        let mut expected = String::new();
        reader.read_to_string(&mut expected).unwrap();
        let expected: Vec<String> = expected
            .split_terminator("\n")
            .map(|s| s.to_owned())
            .collect();
        assert_eq!(parsed, expected);
    }
}
