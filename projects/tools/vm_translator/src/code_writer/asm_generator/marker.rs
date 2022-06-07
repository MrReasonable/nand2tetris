use crate::code_writer::label_generator::LabelGenerator;

use super::flatten;

pub(crate) fn terminate(label_generator: &mut LabelGenerator) -> Vec<String> {
    let end_lbl = label_generator.generate();
    flatten(vec![
        label(&end_lbl),
        vec![format!("@{}", end_lbl), "0;JMP".to_owned()],
    ])
}

pub(crate) fn label(label: &str) -> Vec<String> {
    vec![format!("({})", label)]
}
