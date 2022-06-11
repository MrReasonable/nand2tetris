use crate::{code_writer::label_generator::LabelGenerator, parser::Arithmetic};

use super::{
    flatten,
    flow::{jmp, JmpCmd},
    label,
    register::{set_a_reg_to_pointer, set_alias, CmpVal},
    stack::{dec_stack_pointer, inc_stack_pointer, pop_and_prep_stack, push_d_reg_to_stack},
};

#[derive(Debug)]
enum Cmp {
    Eq,
    Lt,
    Gt,
}

const ADD_SYMBOL: char = '+';
const NEG_SYMBOL: char = '-';
const NOT_SYMBOL: char = '!';
const AND_SYMBOL: char = '&';
const OR_SYMBOL: char = '|';

pub(crate) fn arithmetic(arr: Arithmetic, label_generator: &mut LabelGenerator) -> Vec<String> {
    match arr {
        Arithmetic::Add => bin_math_to_asm(ADD_SYMBOL),
        Arithmetic::Sub => bin_math_to_asm(NEG_SYMBOL),
        Arithmetic::And => bin_math_to_asm(AND_SYMBOL),
        Arithmetic::Or => bin_math_to_asm(OR_SYMBOL),
        Arithmetic::Neg => uni_math_to_asm(NEG_SYMBOL),
        Arithmetic::Not => uni_math_to_asm(NOT_SYMBOL),
        Arithmetic::Eq => cmp_math_to_asm(Cmp::Eq, label_generator),
        Arithmetic::Gt => cmp_math_to_asm(Cmp::Gt, label_generator),
        Arithmetic::Lt => cmp_math_to_asm(Cmp::Lt, label_generator),
    }
}

fn bin_math_to_asm(symbol: char) -> Vec<String> {
    flatten(vec![
        pop_and_prep_stack(),
        vec![format!("M=M{}D", symbol)],
        inc_stack_pointer(),
    ])
}

fn uni_math_to_asm(symbol: char) -> Vec<String> {
    flatten(vec![
        dec_stack_pointer(),
        set_a_reg_to_pointer(),
        vec![format!("M={}M", symbol)],
        inc_stack_pointer(),
    ])
}

fn cmp_math_to_asm(cmp: Cmp, label_generator: &mut LabelGenerator) -> Vec<String> {
    let jmp_cmd = match cmp {
        Cmp::Eq => JmpCmd::Jeq,
        Cmp::Lt => JmpCmd::Jlt,
        Cmp::Gt => JmpCmd::Jgt,
    };
    let true_lbl = label_generator.generate();
    let false_lbl = label_generator.generate();

    flatten(vec![
        pop_and_prep_stack(),
        vec!["D=M-D".to_owned()],
        set_alias(&true_lbl),
        jmp(jmp_cmd, CmpVal::D),
        vec!["D=0".to_owned()],
        set_alias(&false_lbl),
        jmp(JmpCmd::Jmp, CmpVal::Zero),
        label(&true_lbl),
        vec!["D=-1".to_owned()],
        label(&false_lbl),
        push_d_reg_to_stack(),
    ])
}
