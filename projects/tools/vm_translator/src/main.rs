use std::error::Error;

use clap::Parser;
use vm_translator::translator::translate;

///A translator for the Jack VM to Hack assembly language from the nand-to-tetris course
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(name = "input file or directory")]
    input_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    Ok(translate(&args.input_path)?)
}
