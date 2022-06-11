use std::fmt::Display;

use crate::parser::Flow;

use super::{
    flatten, label,
    register::{set_alias, CmpVal},
    stack::pop_stack_to_d_reg,
};

pub(crate) enum JmpCmd {
    Jeq,
    Jlt,
    Jgt,
    Jmp,
}

impl Display for JmpCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JmpCmd::Jeq => "JEQ",
                JmpCmd::Jlt => "JLT",
                JmpCmd::Jgt => "JGT",
                JmpCmd::Jmp => "JMP",
            }
        )
    }
}

pub(crate) fn flow(flow_cmd: Flow) -> Vec<String> {
    match flow_cmd {
        Flow::Label(ref l) => label(l),
        Flow::Goto(goto_type, ref l) => match goto_type {
            crate::parser::Goto::Direct => goto(l),
            crate::parser::Goto::Conditional => if_goto(l),
        },
    }
}

fn goto(label: &str) -> Vec<String> {
    flatten(vec![set_alias(label), jmp(JmpCmd::Jmp, CmpVal::Zero)])
}

fn if_goto(label: &str) -> Vec<String> {
    flatten(vec![
        pop_stack_to_d_reg(),
        set_alias(label),
        jmp(JmpCmd::Jgt, CmpVal::D),
        jmp(JmpCmd::Jlt, CmpVal::D),
    ])
}

pub(super) fn jmp(jmp_cmd: JmpCmd, cmp_val: CmpVal) -> Vec<String> {
    vec![format!("{};{}", cmp_val, jmp_cmd)]
}
