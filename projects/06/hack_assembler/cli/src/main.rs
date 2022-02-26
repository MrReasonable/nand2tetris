use clap::{Parser};
use std::error::Error;
use std::io::Error as IoError;

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    file: String
}

fn main() -> Result<(), Box<dyn Error>>{
    let cli = Cli::parse();
    // assemble(cli.file)?;
    Ok(())
}

// fn assemble(path: String) -> Result<AsmParser, IoError> {
//     let parser = AsmParser::new(&path)?;
//     Ok(parser)
// }