use std::fmt::Display;

use super::flatten;

pub(super) enum Reg {
    A,
    D,
    Mem,
}

pub(super) enum CmpVal {
    A,
    D,
    Mem,
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
                CmpVal::A => "A",
                CmpVal::D => "D",
                CmpVal::Mem => "M",
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

pub(super) fn set_d_reg_to_alias(alias: &str) -> Vec<String> {
    flatten(vec![set_alias(alias), set_d_reg_to_mem()])
}

pub(super) fn set_a_reg_to_pointer() -> Vec<String> {
    set_reg_eq(Reg::A, Reg::Mem)
}

pub(super) fn set_a_reg_to_d_reg() -> Vec<String> {
    set_reg_eq(Reg::A, Reg::D)
}

pub(super) fn set_d_reg_to_constant(value: u16) -> Vec<String> {
    flatten(vec![set_a_reg_to_constant(value), set_d_reg_to_a_reg()])
}

pub(super) fn set_a_reg_to_constant(value: u16) -> Vec<String> {
    vec![format!("@{}", value)]
}

pub(super) fn set_d_reg_to_a_reg() -> Vec<String> {
    set_reg_eq(Reg::D, Reg::A)
}

pub(super) fn set_d_reg_to_mem() -> Vec<String> {
    set_reg_eq(Reg::D, Reg::Mem)
}

pub(super) fn set_mem_to_d_reg() -> Vec<String> {
    set_reg_eq(Reg::Mem, Reg::D)
}

fn set_reg_eq(a: Reg, b: Reg) -> Vec<String> {
    vec![format!("{}={}", a, b)]
}
