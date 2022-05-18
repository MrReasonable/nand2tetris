use std::fmt::Display;

use crate::symbol_table::HackMemSize;

#[derive(Debug, Clone, PartialEq)]
pub enum AInstruction {
    RawAddr(HackMemSize),
    Alias(String),
}

impl Display for AInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
