use std::{error::Error, path::Path};

use clap::Parser;
use vm_translator::translator::{create_code_writer, translate};

///A translator for the Jack VM to Hack assembly language from the nand-to-tetris course
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(name = "input file or directory")]
    input_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut code_writer = create_code_writer(Path::new(&args.input_path))?;
    Ok(translate(&args.input_path, &mut code_writer)?)
}
