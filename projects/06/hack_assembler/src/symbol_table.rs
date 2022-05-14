use std::collections::HashMap;

use thiserror::Error;

pub type HackInstSize = u16;
pub type HackMemSize = u16;
pub type HackRomSize = u16;
const START_ALIAS_ADDRESS: HackMemSize = 0x0010;
const PREDEF_ALIASES: [(&str, HackMemSize); 23] = [
    ("SP", 0x0),
    ("LCL", 0x1),
    ("ARG", 0x2),
    ("THIS", 0x3),
    ("THAT", 0x4),
    ("R0", 0x0),
    ("R1", 0x1),
    ("R2", 0x2),
    ("R3", 0x3),
    ("R4", 0x4),
    ("R5", 0x5),
    ("R6", 0x6),
    ("R7", 0x7),
    ("R8", 0x8),
    ("R9", 0x9),
    ("R10", 0xa),
    ("R11", 0xb),
    ("R12", 0xc),
    ("R13", 0xd),
    ("R14", 0xe),
    ("R15", 0xf),
    ("SCREEN", SCREEN_MEM),
    ("KBD", KBD_MEM),
];
const SCREEN_MEM: HackMemSize = 0x4000;
const KBD_MEM: HackMemSize = 0x6000;

const DEST_INSTR: [(&str, HackInstSize); 3] = [("M", 0b001), ("D", 0b010), ("A", 0b100)];

const JGT: HackInstSize = 0b001;
const JEQ: HackInstSize = 0b010;
const JLT: HackInstSize = 0b100;
const JMP_INSTR: [(&str, HackInstSize); 7] = [
    ("JGT", JGT),
    ("JEQ", JEQ),
    ("JLT", JLT),
    ("JGE", JGT | JEQ),
    ("JLE", JLT | JEQ),
    ("JNE", JLT | JGT),
    ("JMP", JLT | JGT | JEQ),
];

const C6: u16 = 0b0000001;
const C5: u16 = 0b0000010;
const C4: u16 = 0b0000100;
const C3: u16 = 0b0001000;
const C2: u16 = 0b0010000;
const C1: u16 = 0b0100000;
const A_BIT: u16 = 0b1000000;

const COMP_INSTR: [(&str, HackInstSize); 28] = [
    ("0", C5 | C3 | C1),
    ("1", C6 | C5 | C4 | C3 | C2 | C1),
    ("-1", C5 | C3 | C2 | C1),
    ("D", C4 | C3),
    ("A", C2 | C1),
    ("!D", C6 | C4 | C3),
    ("!A", C6 | C2 | C1),
    ("-D", C6 | C5 | C4 | C3),
    ("-A", C6 | C5 | C2 | C1),
    ("D+1", C6 | C5 | C4 | C3 | C2),
    ("A+1", C6 | C5 | C4 | C2 | C1),
    ("D-1", C5 | C4 | C3),
    ("A-1", C5 | C2 | C1),
    ("D+A", C5),
    ("D-A", C6 | C5 | C2),
    ("A-D", C6 | C5 | C4),
    ("D&A", 0),
    ("D|A", C6 | C4 | C2),
    ("M", A_BIT | C2 | C1),
    ("!M", A_BIT | C6 | C2 | C1),
    ("-M", A_BIT | C6 | C5 | C2 | C1),
    ("M+1", A_BIT | C6 | C5 | C4 | C2 | C1),
    ("M-1", A_BIT | C5 | C2 | C1),
    ("D+M", A_BIT | C5),
    ("D-M", A_BIT | C6 | C5 | C2),
    ("M-D", A_BIT | C6 | C5 | C4),
    ("D&M", A_BIT),
    ("D|M", A_BIT | C6 | C4 | C2),
];

pub const START_CMP_INSTR: u16 = 0b1110000000000000;

#[derive(Debug, Error)]
pub enum SymbolTableError {
    #[error("already set error")]
    AlreadySetErr,
}

#[derive(Debug)]
pub struct SymbolTable {
    aliases: HashMap<String, HackMemSize>,
    next_mem_allocation: HackMemSize,
    labels: HashMap<String, HackRomSize>,
    dest_instr: HashMap<String, HackInstSize>,
    jmp_instr: HashMap<String, HackInstSize>,
    comp_instr: HashMap<String, HackInstSize>,
}

impl<'a> Default for SymbolTable {
    fn default() -> Self {
        SymbolTable::new()
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            aliases: SymbolTable::init_predefined(PREDEF_ALIASES),
            next_mem_allocation: START_ALIAS_ADDRESS,
            labels: HashMap::new(),
            dest_instr: SymbolTable::init_predefined(DEST_INSTR),
            jmp_instr: SymbolTable::init_predefined(JMP_INSTR),
            comp_instr: SymbolTable::init_predefined(COMP_INSTR),
        }
    }

    fn init_predefined<const N: usize>(
        predef: [(&str, HackInstSize); N],
    ) -> HashMap<String, HackInstSize> {
        predef
            .into_iter()
            .map(|val| (val.0.to_string(), val.1))
            .collect()
    }

    pub fn add_alias(&mut self, alias: String) -> Result<HackMemSize, SymbolTableError> {
        if self.aliases.contains_key(&alias) {
            return Err(SymbolTableError::AlreadySetErr);
        }

        let location = self.next_mem_allocation;
        self.next_mem_allocation += 1;
        match self.aliases.insert(alias, location) {
            None => Ok(location),
            Some(_) => Err(SymbolTableError::AlreadySetErr),
        }
    }

    pub fn get_addr(&self, alias: &str) -> Option<HackMemSize> {
        self.aliases.get(alias).copied()
    }

    pub fn add_label(
        &mut self,
        label: String,
        line_no: HackRomSize,
    ) -> Result<HackRomSize, SymbolTableError> {
        if self.labels.contains_key(&label) {
            return Err(SymbolTableError::AlreadySetErr);
        }

        match self.labels.insert(label, line_no) {
            None => Ok(line_no),
            Some(_) => Err(SymbolTableError::AlreadySetErr),
        }
    }

    pub fn get_line_no(&self, label: &str) -> Option<HackRomSize> {
        self.labels.get(label).copied()
    }

    pub fn get_jmp_instr(&self, jmp_instr: &str) -> Option<HackInstSize> {
        self.jmp_instr.get(jmp_instr).copied()
    }

    pub fn get_dest_instr(&self, dest_instr: &str) -> Option<HackInstSize> {
        let dest_bits = dest_instr.chars().map(|dest| {
            let tmp = dest.to_string();
            self.dest_instr.get(&tmp[..]).copied()
        });

        let result =
            dest_bits.reduce(
                |accum: Option<u16>, dest: Option<u16>| match (accum, dest) {
                    (None, _) => None,
                    (_, None) => None,
                    (Some(a), Some(b)) => Some(a | b),
                },
            );

        match result {
            None => None,
            Some(None) => None,
            Some(a) => a,
        }
    }

    pub fn get_comp_instr(&self, comp_instr: &str) -> Option<HackInstSize> {
        self.comp_instr.get(comp_instr).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn it_inits_with_predefined_symbols() {
        let symbol_table = SymbolTable::new();
        for (alias, location) in PREDEF_ALIASES {
            assert!(symbol_table.aliases.contains_key(alias));
            assert_eq!(*symbol_table.aliases.get(alias).unwrap(), location);
        }
    }

    #[test]
    fn it_does_not_permit_redeclaration_of_symbols() {
        let mut symbol_table = SymbolTable::new();
        let result = symbol_table.add_alias(PREDEF_ALIASES[0].0.to_string());
        assert_matches!(result, Err(SymbolTableError::AlreadySetErr));
    }

    #[test]
    fn it_allocates_0x0010_for_first_alias_address() {
        let mut symbol_table = SymbolTable::new();
        let result = symbol_table.add_alias("test".to_string());
        assert_matches!(result, Ok(0x0010));
    }

    #[test]
    fn it_allocates_incremental_locations_for_subsequent_aliases() {
        let mut symbol_table = SymbolTable::new();
        let result = symbol_table.add_alias("test1".to_string());
        assert_matches!(result, Ok(0x0010));
        let result = symbol_table.add_alias("test2".to_string());
        assert_matches!(result, Ok(0x0011));
        let result = symbol_table.add_alias("test3".to_string());
        assert_matches!(result, Ok(0x0012));
    }

    #[test]
    fn it_returns_allocated_address_for_aliases() {
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_alias("test1".to_string()).unwrap();
        assert_matches!(symbol_table.get_addr("test1"), Some(0x0010));
        symbol_table.add_alias("test2".to_string()).unwrap();
        assert_matches!(symbol_table.get_addr("test1"), Some(0x0010));
        assert_matches!(symbol_table.get_addr("test2"), Some(0x0011));
    }

    #[test]
    fn it_returns_none_for_unrecognised_alias() {
        let mut symbol_table = SymbolTable::new();
        assert_matches!(symbol_table.get_addr("test1"), None);
        symbol_table.add_alias("test1".to_string()).unwrap();
        assert_matches!(symbol_table.get_addr("test1"), Some(0x0010));
    }

    #[test]
    fn it_does_not_permit_redecleration_of_labels() {
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_label("test1".to_string(), 1).unwrap();
        assert_matches!(
            symbol_table.add_label("test1".to_string(), 2),
            Err(SymbolTableError::AlreadySetErr)
        );
        assert_eq!(*symbol_table.labels.get("test1").unwrap(), 1);
    }

    #[test]
    fn it_sets_label_to_supplied_line_no() {
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_label("test1".to_string(), 1).unwrap();
        symbol_table.add_label("test2".to_string(), 3).unwrap();
        assert_eq!(symbol_table.get_line_no("test1"), Some(1));
        assert_eq!(symbol_table.get_line_no("test2"), Some(3));
    }

    #[test]
    fn it_keeps_labels_and_aliases_seperate() {
        let mut symbol_table = SymbolTable::new();
        symbol_table.add_label("SCREEN".to_string(), 0x1).unwrap();
        assert_eq!(symbol_table.get_addr("SCREEN"), Some(SCREEN_MEM));
        symbol_table.add_alias("test1".to_string()).unwrap();
        symbol_table.add_label("test1".to_string(), 0x1).unwrap();

        assert_eq!(symbol_table.get_addr("test1"), Some(START_ALIAS_ADDRESS));
        assert_matches!(symbol_table.get_line_no("test1"), Some(0x1));
    }

    #[test]
    fn it_provides_bits_for_jump_instructions() {
        let symbol_table = SymbolTable::new();
        assert_eq!(symbol_table.get_jmp_instr("JGT"), Some(0b001));
        assert_eq!(symbol_table.get_jmp_instr("JEQ"), Some(0b010));
        assert_eq!(symbol_table.get_jmp_instr("JGE"), Some(0b011));
        assert_eq!(symbol_table.get_jmp_instr("JLT"), Some(0b100));
        assert_eq!(symbol_table.get_jmp_instr("JNE"), Some(0b101));
        assert_eq!(symbol_table.get_jmp_instr("JLE"), Some(0b110));
        assert_eq!(symbol_table.get_jmp_instr("JMP"), Some(0b111));
    }

    #[test]
    fn it_provides_bits_for_dest_instructions() {
        let symbol_table = SymbolTable::new();
        println!("{:?}", symbol_table.dest_instr);
        assert_eq!(symbol_table.get_dest_instr("M"), Some(0b001));
        assert_eq!(symbol_table.get_dest_instr("D"), Some(0b010));
        assert_eq!(symbol_table.get_dest_instr("MD"), Some(0b011));
        assert_eq!(symbol_table.get_dest_instr("A"), Some(0b100));
        assert_eq!(symbol_table.get_dest_instr("AM"), Some(0b101));
        assert_eq!(symbol_table.get_dest_instr("AD"), Some(0b110));
        assert_eq!(symbol_table.get_dest_instr("AMD"), Some(0b111));
    }
}
