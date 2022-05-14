use std::io::{BufReader, Read};

use crate::{
    instructions::{AInstruction, CInstruction},
    symbol_table::{HackRomSize, SymbolTable, SymbolTableError, START_CMP_INSTR},
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
    #[error("non-compilable token: {0}")]
    NonCompilableToken(Token),
    #[error("address not found for alias: {0}")]
    AliasNotFound(String),
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
    let bin = convert_to_bin(tokens, symbols)?;
    println!("Bin: {:?}", bin);
    Ok(())
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
                Token::AInstruction(ref alias) => {
                    match alias {
                        AInstruction::Alias(alias)
                            if symbols.get_addr(alias).is_none()
                                && symbols.get_line_no(alias).is_none() =>
                        {
                            symbols.add_alias(alias.clone())?;
                        }
                        _ => {}
                    }
                    tokens.push(token);
                }
                Token::CInstruction(_) => tokens.push(token),
            }
        }
    }
    Ok((symbols, tokens))
}

fn convert_to_bin(tokens: Vec<Token>, symbols: SymbolTable) -> Result<Vec<u16>, ParseError> {
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
                        Err(ParseError::AliasNotFound(alias.clone()))
                    }
                }
            },
            Token::CInstruction(cinstr) => Ok(CInstWithSymbols(cinstr, &symbols).into()),
            token => Err(ParseError::NonCompilableToken(token.clone())),
        })
        .collect()
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
