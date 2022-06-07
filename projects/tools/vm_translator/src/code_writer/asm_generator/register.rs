use super::flatten;

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
    vec!["A=M".to_owned()]
}

pub(super) fn set_a_reg_to_d_reg() -> Vec<String> {
    vec!["A=D".to_owned()]
}

pub(super) fn set_d_reg_to_constant(value: u16) -> Vec<String> {
    flatten(vec![set_a_reg_to_constant(value), set_d_reg_to_a_reg()])
}

pub(super) fn set_a_reg_to_constant(value: u16) -> Vec<String> {
    vec![format!("@{}", value)]
}

pub(super) fn set_d_reg_to_a_reg() -> Vec<String> {
    vec!["D=A".to_owned()]
}

pub(super) fn set_d_reg_to_mem() -> Vec<String> {
    vec!["D=M".to_owned()]
}

pub(super) fn set_mem_to_d_reg() -> Vec<String> {
    vec!["M=D".to_owned()]
}
