use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lazy_static::lazy_static;

use crate::{
    code_writer::reg_mgr::{Reg, RegMgr, RegMgrError},
    parser::{PopSegment, PushSegment, Segment},
};

use super::{
    flatten,
    register::{
        set_a_reg_to_alias, set_a_reg_to_constant, set_a_reg_to_d_reg, set_alias,
        set_d_reg_to_alias, set_d_reg_to_constant, set_d_reg_to_mem, set_mem_to_d_reg,
    },
    stack::{pop_stack_to_d_reg, push_d_reg_to_stack},
};

lazy_static! {
    static ref PUSH_MEM_MAP: HashMap<PushSegment, &'static str> = {
        let mut m = HashMap::new();
        m.insert(PushSegment::Local, "LCL");
        m.insert(PushSegment::Argument, "ARG");
        m.insert(PushSegment::This, "THIS");
        m.insert(PushSegment::That, "THAT");
        m.insert(PushSegment::Constant, "SP");
        m
    };
}

lazy_static! {
    static ref POP_MEM_MAP: HashMap<PopSegment, &'static str> = {
        let mut m = HashMap::new();
        m.insert(PopSegment::Local, "LCL");
        m.insert(PopSegment::Argument, "ARG");
        m.insert(PopSegment::This, "THIS");
        m.insert(PopSegment::That, "THAT");
        m
    };
}

const TMP_BASE_ADDR: u16 = 5;
const AVAILABLE_TMP_BLOCKS: u16 = 8;

pub(crate) struct MemCmdWriter {
    namespace: String,
    gen_purp_reg: Rc<RefCell<RegMgr>>,
}

#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("Temp error: {0}")]
    TempError(#[from] RegMgrError),
    #[error("Memory out of bounds: {0} is out of bounds of segment {1}")]
    OutOfBoundsPushError(u16, PushSegment),
    #[error("Memory out of bounds: {0} is out of bounds of segment {1}")]
    OutOfBoundsPopError(u16, PopSegment),
}

impl MemCmdWriter {
    pub fn new(namespace: String, gen_purp_reg: Rc<RefCell<RegMgr>>) -> Self {
        Self {
            namespace: namespace.to_uppercase(),
            gen_purp_reg,
        }
    }

    pub fn push_to_stack(
        &self,
        segment: PushSegment,
        idx: u16,
    ) -> Result<Vec<String>, MemoryError> {
        Ok(flatten(vec![
            match segment {
                PushSegment::Constant => set_d_reg_to_constant(idx),
                PushSegment::Static => flatten(vec![set_d_reg_to_alias(
                    format!("{}.{}", self.namespace, idx).as_str(),
                )]),
                PushSegment::Temp => {
                    if idx > AVAILABLE_TMP_BLOCKS - 1 {
                        Err(MemoryError::OutOfBoundsPushError(idx, segment))?
                    } else {
                        flatten(vec![
                            set_a_reg_to_constant(TMP_BASE_ADDR + idx),
                            set_d_reg_to_mem(),
                        ])
                    }
                }
                PushSegment::Pointer => {
                    if idx > 1 {
                        Err(MemoryError::OutOfBoundsPushError(idx, segment))?
                    } else {
                        let segment = if idx == 0 {
                            PushSegment::This.into()
                        } else {
                            PushSegment::That.into()
                        };
                        set_d_reg_to_alias(get_segment_alias(&segment))
                    }
                }
                segment => flatten(vec![
                    self.set_d_reg_to_segment_idx(&segment.into(), idx),
                    set_a_reg_to_d_reg(),
                    set_d_reg_to_mem(),
                ]),
            },
            push_d_reg_to_stack(),
        ]))
    }

    pub(crate) fn pop_stack_to(
        &self,
        segment: PopSegment,
        idx: u16,
    ) -> Result<Vec<String>, MemoryError> {
        let tmp = self.gen_purp_reg.borrow_mut().next()?;
        Ok(flatten(vec![match segment {
            PopSegment::Static => flatten(vec![
                pop_stack_to_d_reg(),
                set_alias(format!("{}.{}", self.namespace, idx).as_str()),
                set_mem_to_d_reg(),
            ]),
            PopSegment::Temp => {
                if idx > AVAILABLE_TMP_BLOCKS - 1 {
                    Err(MemoryError::OutOfBoundsPopError(idx, segment))?
                } else {
                    flatten(vec![
                        pop_stack_to_d_reg(),
                        set_a_reg_to_constant(TMP_BASE_ADDR + idx),
                        set_mem_to_d_reg(),
                    ])
                }
            }
            PopSegment::Pointer => {
                if idx > 1 {
                    Err(MemoryError::OutOfBoundsPopError(idx, segment))?
                } else {
                    let segment = if idx == 0 {
                        PushSegment::This.into()
                    } else {
                        PushSegment::That.into()
                    };
                    flatten(vec![
                        pop_stack_to_d_reg(),
                        set_alias(get_segment_alias(&segment)),
                        set_mem_to_d_reg(),
                    ])
                }
            }
            segment => flatten(vec![
                self.set_d_reg_to_segment_idx(&segment.into(), idx),
                set_alias(format!("{}", tmp).as_str()),
                set_mem_to_d_reg(),
                pop_stack_to_d_reg(),
                self.copy_d_reg_to_mem(tmp),
            ]),
        }]))
    }

    fn set_d_reg_to_segment_idx(&self, segment: &Segment, idx: u16) -> Vec<String> {
        let asm = flatten(vec![
            set_d_reg_to_alias(get_segment_alias(&segment)),
            if idx > 0 {
                flatten(vec![set_a_reg_to_constant(idx), vec![format!("D=D+A")]])
            } else {
                vec![]
            },
        ]);
        asm
    }

    fn copy_d_reg_to_mem(&self, reg: Reg) -> Vec<String> {
        flatten(vec![
            set_a_reg_to_alias(format!("{}", reg).as_str()),
            set_mem_to_d_reg(),
        ])
    }
}

pub(super) fn get_segment_alias(segment: &Segment) -> &str {
    match segment {
        Segment::PushSegment(p) => *PUSH_MEM_MAP.get(p).unwrap(),
        Segment::PopSegment(p) => *POP_MEM_MAP.get(p).unwrap(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    fn make_mem_cmd_writer() -> MemCmdWriter {
        MemCmdWriter::new(
            "test".to_string(),
            Rc::new(RefCell::new(RegMgr::new(5, 8).unwrap())),
        )
    }

    #[test_case(
        PushSegment::Temp,
        AVAILABLE_TMP_BLOCKS;
        "temp"
    )]
    #[test_case(
        PushSegment::Pointer,
        2;
        "pointer"
    )]
    fn it_should_raise_error_when_pushing_out_of_bounds_segment(segment: PushSegment, idx: u16) {
        let cmd_writer = make_mem_cmd_writer();
        assert!(cmd_writer.push_to_stack(segment, idx).is_err());
    }

    #[test_case(
        PopSegment::Temp,
        AVAILABLE_TMP_BLOCKS;
        "temp"
    )]
    #[test_case(
        PopSegment::Pointer,
        2;
        "pointer"
    )]
    fn it_should_raise_error_when_popping_out_of_bounds_segment(segment: PopSegment, idx: u16) {
        let cmd_writer = make_mem_cmd_writer();
        assert!(cmd_writer.pop_stack_to(segment, idx).is_err());
    }
}
