use crate::{code_writer::label_manager::LabelManager, parser::Marker};

use super::{flatten, register::set_d_reg_to_constant, stack::push_d_reg_to_stack, MemoryError};

#[derive(thiserror::Error, Debug)]
pub enum MarkerError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),
}

pub(crate) fn marker(marker_cmd: Marker, label_manager: &mut LabelManager) -> Vec<String> {
    match marker_cmd {
        Marker::Label(ref l) => label(l),
        Marker::Function(ref name, local_count) => {
            let ret = function(name, local_count);
            label_manager.start_function(name);
            ret
        }
    }
}

pub(crate) fn label(label: &str) -> Vec<String> {
    vec![format!("({})", label)]
}

fn function(name: &str, local_count: u8) -> Vec<String> {
    flatten(vec![label(name), initialize_locals(local_count)])
}

fn initialize_locals(local_count: u8) -> Vec<String> {
    if local_count > 0 {
        (0..local_count)
            .flat_map(|_| flatten(vec![set_d_reg_to_constant(0), push_d_reg_to_stack()]))
            .collect()
    } else {
        vec![]
    }
}
