use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct CInstruction {
    dest: Option<String>,
    comp: String,
    jump: Option<String>,
}

impl CInstruction {
    pub fn new(dest: Option<String>, comp: String, jump: Option<String>) -> CInstruction {
        CInstruction { dest, comp, jump }
    }

    pub fn dest(&self) -> Option<&String> {
        self.dest.as_ref()
    }

    pub fn comp(&self) -> &str {
        self.comp.as_ref()
    }

    pub fn jump(&self) -> Option<&String> {
        self.jump.as_ref()
    }
}

impl Display for CInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}
