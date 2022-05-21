use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Seek, Write},
};

use tempfile::tempfile;

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

pub fn parse<R, W>(source: &mut BufReader<R>, dest: &mut BufWriter<W>) -> Result<(), ParseError>
where
    R: Read,
    W: Write,
{
    let (symbols, mut parsed_source) = first_pass(source)?;
    Ok(convert_to_bin(symbols, &mut parsed_source, dest)?)
}

fn first_pass<R: Read>(
    code: &mut BufReader<R>,
) -> Result<(SymbolTable, BufReader<File>), ParseError> {
    let mut symbols = SymbolTable::new();
    let mut tmp_file = tempfile()?;
    let mut line = String::new();
    let mut line_counter = 0;

    while code.read_line(&mut line)? > 0 {
        let trimmed_line = line.trim();
        if let Some(token) = tokenize(trimmed_line)? {
            match token {
                Token::Label(ref label) => {
                    symbols.add_label(label.clone(), (line_counter) as HackRomSize)?;
                }
                Token::CInstruction(_) | Token::AInstruction(_) => {
                    writeln!(tmp_file, "{}", trimmed_line)?;
                    line_counter += 1;
                }
            }
        }
        line = "".to_owned();
    }
    tmp_file.rewind()?;
    Ok((symbols, BufReader::new(tmp_file)))
}

fn convert_to_bin<W: Write>(
    mut symbols: SymbolTable,
    tokens: &mut BufReader<File>,
    target: &mut BufWriter<W>,
) -> Result<(), ParseError> {
    let mut line = String::new();
    while tokens.read_line(&mut line)? > 0 {
        let trimmed_line = line.trim();
        if let Some(code) = tokenize(trimmed_line)? {
            let token = match code {
                Token::AInstruction(a) => match a {
                    AInstruction::RawAddr(addr) => Ok(addr),
                    AInstruction::Alias(ref alias) => {
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
                Token::CInstruction(ref cinstr) => Ok(CInstWithSymbols(cinstr, &symbols).into()),
                token => Err(ParseError::NonCompilableToken(token.clone())),
            }?;
            let bin = bin_string(token);
            writeln!(target, "{}", bin)?;
        }
        line = "".to_owned();
    }
    Ok(())
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
        let mut writer = BufWriter::new(Vec::new());
        parse(&mut reader, &mut writer).unwrap();
        let mut reader = setup(Path::new("./test_files/test-cmp.hack"));
        let mut expected = String::new();
        reader.read_to_string(&mut expected).unwrap();
        let actual = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(actual, expected);
    }
}
