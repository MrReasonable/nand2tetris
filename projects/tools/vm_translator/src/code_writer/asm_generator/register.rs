use std::fmt::Display;

use super::flatten;

pub(super) enum Reg {
    A,
    D,
    Mem,
}

pub(super) enum CmpVal {
    D,
    Zero,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg::A => "A",
                Reg::D => "D",
                Reg::Mem => "M",
            }
        )
    }
}

impl Display for CmpVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CmpVal::D => "D",
                CmpVal::Zero => "0",
            },
        )
    }
}

pub(super) fn set_alias(alias: &str) -> Vec<String> {
    vec![format!("@{}", alias)]
}

pub(super) fn set_a_reg_to_alias(alias: &str) -> Vec<String> {
    flatten(vec![set_alias(alias), set_a_reg_to_pointer()])
}

pub(super) fn set_d_reg_to_alias(alias: &str, relative: Option<i16>) -> Vec<String> {
    match relative {
        Some(idx) if idx == -1 => flatten(vec![set_alias(alias), vec!["D=M-1".to_owned()]]),
        Some(idx) if idx == 1 => flatten(vec![set_alias(alias), vec!["D=M+1".to_owned()]]),
        Some(idx) if idx != 0 => flatten(vec![
            set_d_reg_to_alias(alias, None),
            set_a_reg_to_constant(idx.abs()),
            if idx > 0 {
                vec!["D=D+A".to_owned()]
            } else {
                vec!["D=D-A".to_owned()]
            },
        ]),
        _ => flatten(vec![set_alias(alias), set_d_reg_to_mem()]),
    }
}

pub(super) fn set_a_reg_to_pointer() -> Vec<String> {
    set_reg_to(Reg::A, Reg::Mem)
}

pub(super) fn set_d_reg_to_constant(value: i16) -> Vec<String> {
    if value == 0 {
        vec![format!("D=0")]
    } else if value == 1 {
        vec![format!("D=1")]
    } else if value == -1 {
        vec![format!("D=-1")]
    } else {
        flatten(vec![set_a_reg_to_constant(value), set_d_reg_to_a_reg()])
    }
}

pub(super) fn set_a_reg_to_constant(value: i16) -> Vec<String> {
    vec![format!("@{}", value)]
}

pub(super) fn set_a_reg_to_address(value: u16) -> Vec<String> {
    vec![format!("@{}", value)]
}

pub(super) fn set_d_reg_to_a_reg() -> Vec<String> {
    set_reg_to(Reg::D, Reg::A)
}

pub(super) fn set_d_reg_to_mem() -> Vec<String> {
    set_reg_to(Reg::D, Reg::Mem)
}

pub(super) fn set_mem_to_d_reg() -> Vec<String> {
    set_reg_to(Reg::Mem, Reg::D)
}

fn set_reg_to(a: Reg, b: Reg) -> Vec<String> {
    vec![format!("{}={}", a, b)]
}

pub(super) fn set_mem_at_alias_to_d_reg(alias: &str) -> Vec<String> {
    flatten(vec![set_a_reg_to_alias(alias), set_mem_to_d_reg()])
}
