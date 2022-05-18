use std::io::{self, BufReader, BufWriter, Read, Write};

use thiserror::Error;

use crate::parser::{parse, ParseError};

#[derive(Debug, Error)]
pub enum AssemblerError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("file error: {0}")]
    IoError(#[from] io::Error),
}

pub fn assemble<R: Read, W: Write>(
    mut reader: BufReader<R>,
    mut writer: BufWriter<W>,
) -> Result<(), AssemblerError> {
    let asm = parse(&mut reader)?;
    for instr in asm {
        writeln!(writer, "{}", instr)?;
    }
    Ok(())
}
