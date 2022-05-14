use clap::Parser;
use hack_assembler::{tokenize, FileReader, Readable};
use std::error::Error;

///An assembler for the Hack assembly languagae from the nand-to-tetris course
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(name = "input file")]
    in_file: String,
    #[clap(name = "output file")]
    out_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let reader = FileReader::new(args.in_file);
    let contents = reader.read()?;
    let lines: Vec<&str> = contents.lines().collect();
    let tokens = tokenize(lines[0])?;
    dbg!(tokens.unwrap());
    Ok(())
}

// fn assemble(path: String) -> Result<AsmParser, IoError> {
//     let parser = AsmParser::new(&path)?;
//     Ok(parser)
// }
