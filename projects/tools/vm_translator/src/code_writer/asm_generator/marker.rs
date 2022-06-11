use crate::code_writer::label_generator::LabelGenerator;

use super::{
    flatten,
    flow::{jmp, JmpCmd},
    register::{set_alias, CmpVal},
};

pub(crate) fn terminate(label_generator: &mut LabelGenerator) -> Vec<String> {
    let end_lbl = label_generator.generate();
    flatten(vec![
        label(&end_lbl),
        set_alias(&end_lbl),
        jmp(JmpCmd::Jmp, CmpVal::Zero),
    ])
}

pub(crate) fn label(label: &str) -> Vec<String> {
    vec![format!("({})", label)]
}
