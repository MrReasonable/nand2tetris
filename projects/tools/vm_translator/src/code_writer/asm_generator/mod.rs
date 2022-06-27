mod arithmetic;
mod flow;
mod marker;
mod memory;
mod register;
mod stack;

pub(super) use arithmetic::arithmetic;
pub(crate) use flow::flow;
pub(crate) use flow::FlowError;
pub(super) use marker::label;
pub(super) use marker::marker;
pub(crate) use memory::MemCmdWriter;
pub(crate) use memory::MemoryError;

fn flatten(asm: Vec<Vec<String>>) -> Vec<String> {
    asm.into_iter().flatten().collect()
}
