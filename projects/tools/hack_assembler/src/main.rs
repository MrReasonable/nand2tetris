use clap::Parser;
use hack_assembler::parse;
use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
};

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
    let mut reader = BufReader::new(File::open(args.in_file)?);
    let mut writer = BufWriter::new(File::create(args.out_file)?);
    parse(&mut reader, &mut writer)?;
    Ok(())
}
