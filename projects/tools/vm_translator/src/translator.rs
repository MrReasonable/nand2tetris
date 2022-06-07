use std::{
    fs::{read_dir, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{
    code_writer::{CodeWriter, CodeWriterError},
    parser::{self, ParseError, ParsedCmd, Parser},
};

#[derive(thiserror::Error, Debug)]
pub enum TranslatorError {
    #[error("invalid path error: {0}")]
    InvalidPathError(PathBuf),
    #[error("file error: {0}")]
    FileError(#[from] io::Error),
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("code writer error: {0}")]
    CodeWriter(#[from] CodeWriterError),
}

pub fn translate<W: Write>(
    input_path: &str,
    code_writer: &mut CodeWriter<W>,
) -> Result<(), TranslatorError> {
    let path = Path::new(input_path);

    if path.is_dir() {
        let mut entries = read_dir(path)?
            .filter_map(|res| match res.map(|entry| entry.path()) {
                Ok(path) => {
                    if let Some("vm") = path.extension().and_then(|p| p.to_str()) {
                        Some(Ok(path))
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();
        for entry in entries {
            parse_file(&*entry, code_writer)?;
        }
    } else {
        parse_file(path, code_writer)?;
    }
    code_writer.write(parser::Command::new("".to_owned(), ParsedCmd::Terminate))?;
    Ok(())
}

pub fn create_code_writer(path: &Path) -> Result<CodeWriter<BufWriter<File>>, TranslatorError> {
    let name = get_path_name(path)?;
    let out_file = File::options()
        .write(true)
        .create(true)
        .open(format!("{}.asm", name))?;
    let out_buffer = BufWriter::new(out_file);
    Ok(CodeWriter::new(out_buffer)?)
}

fn parse_file<W: Write>(
    in_file: &Path,
    code_writer: &mut CodeWriter<W>,
) -> Result<(), TranslatorError> {
    let file = File::open(in_file)?;
    let parser = Parser::new(BufReader::new(file));
    code_writer.set_namespace(get_path_name(in_file)?);
    for command in parser {
        code_writer.write(command?)?
    }
    Ok(())
}

fn get_path_name(path: &Path) -> Result<&'_ str, TranslatorError> {
    path.file_stem()
        .ok_or_else(|| TranslatorError::InvalidPathError(path.to_path_buf()))?
        .to_str()
        .ok_or_else(|| TranslatorError::InvalidPathError(path.to_path_buf()))
}
