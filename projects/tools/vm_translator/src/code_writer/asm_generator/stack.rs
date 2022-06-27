use super::{
    flatten,
    register::{set_a_reg_to_alias, set_a_reg_to_pointer, set_d_reg_to_mem},
};

pub(super) const SEGMENT_STACK: &str = "SP";

pub(super) fn push_d_reg_to_stack() -> Vec<String> {
    flatten(vec![
        set_a_reg_to_alias(SEGMENT_STACK),
        vec!["M=D".to_owned()],
        inc_stack_pointer(),
    ])
}

pub(super) fn pop_stack_to_d_reg() -> Vec<String> {
    flatten(vec![
        dec_stack_pointer(),
        set_a_reg_to_pointer(),
        set_d_reg_to_mem(),
    ])
}

pub(crate) fn pop_and_prep_stack() -> Vec<String> {
    flatten(vec![
        pop_stack_to_d_reg(),
        dec_stack_pointer(),
        set_a_reg_to_pointer(),
    ])
}

pub(super) fn dec_stack_pointer() -> Vec<String> {
    vec!["@SP".to_owned(), "M=M-1".to_owned()]
}

pub(super) fn inc_stack_pointer() -> Vec<String> {
    vec!["@SP".to_owned(), "M=M+1".to_owned()]
}
