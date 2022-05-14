use std::io::{BufReader, Read};

use crate::{
    symbol_table::{HackRomSize, SymbolTable, SymbolTableError},
    tokenizer::{tokenize, Token, TokenError},
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("i/o error: {0}")]
    IoError(std::io::Error),
    #[error("tokenize error: {0}")]
    TokenError(TokenError),
    #[error("symbol table error: {0}")]
    SymbolTableError(SymbolTableError),
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<TokenError> for ParseError {
    fn from(e: TokenError) -> Self {
        Self::TokenError(e)
    }
}

impl From<SymbolTableError> for ParseError {
    fn from(e: SymbolTableError) -> Self {
        Self::SymbolTableError(e)
    }
}

pub fn parse<R>(
    source: &mut BufReader<R>,
    // target: BufWriter<W>,
) -> Result<(), ParseError>
where
    R: Read,
{
    let mut code = String::new();
    source.read_to_string(&mut code)?;
    let (symbols, tokens) = first_pass(&code)?;
    println!("Symbols: {:?}", symbols);
    println!("Tokens: {:?}", tokens);
    Ok(())
}

fn first_pass(code: &str) -> Result<(SymbolTable, Vec<Token>), ParseError> {
    let mut symbols = SymbolTable::new();
    let mut tokens = Vec::new();

    for line in code.lines() {
        if let Some(token) = tokenize(line)? {
            match token {
                Token::Label(ref label) => {
                    symbols.add_label(label.clone(), (tokens.len() + 1) as HackRomSize)?;
                }
                Token::AInstruction(ref alias) => {
                    if symbols.get_addr(alias).is_none() && symbols.get_line_no(alias).is_none() {
                        symbols.add_alias(alias.clone())?;
                    }
                }
                _ => (),
            }
            tokens.push(token);
        }
    }
    Ok((symbols, tokens))
}

fn convert_to_bin(symbols: SymbolTable, tokens: Vec<Token>) -> Result<(), ParseError> {
    let binary = tokens.iter().map(|token| {
        match token {
            Token::Label(l) => ,
            Token::AInstruction(_) => todo!(),
            Token::CInstruction(_) => todo!(),
        }
    }).collect();
    todo!()
}

// fn second_pass(symbols: SymbolTable, tokens: &mut Vec<Token>) -> Result<Vec<u16>, ParseError> {
//     todo!()
// }

#[cfg(test)]
mod test {
    use std::{fs::File, path::Path};

    use super::*;

    fn setup(p: &Path) -> BufReader<File> {
        let reader = File::open(p).unwrap();
        BufReader::new(reader)
    }

    #[test]
    fn compile_max() {
        let mut reader = setup(Path::new("./test_files/Max.asm"));
        // let mem_store = Vec::new();
        // let writer = BufWriter::new(mem_store);
        parse(&mut reader).unwrap();
    }
}
