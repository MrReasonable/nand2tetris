use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lazy_static::lazy_static;

use crate::{
    code_writer::reg_mgr::{RegMgr, RegMgrError},
    parser::Segment,
};

use super::{
    flatten,
    register::{
        set_a_reg_to_address, set_a_reg_to_alias, set_a_reg_to_constant, set_alias,
        set_d_reg_to_alias, set_d_reg_to_constant, set_d_reg_to_mem, set_mem_at_alias_to_d_reg,
        set_mem_to_d_reg,
    },
    stack::{pop_stack_to_d_reg, push_d_reg_to_stack},
};

lazy_static! {
    static ref SEGMENT_MEM_MAP: HashMap<Segment, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Segment::Local, "LCL");
        m.insert(Segment::Argument, "ARG");
        m.insert(Segment::This, "THIS");
        m.insert(Segment::That, "THAT");
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
    Temp(#[from] RegMgrError),
    #[error("Memory out of bounds: {0} is out of bounds of segment {1}")]
    OutOfBounds(u16, Segment),
}

impl MemCmdWriter {
    pub fn new(namespace: String, gen_purp_reg: Rc<RefCell<RegMgr>>) -> Self {
        Self {
            namespace: namespace.to_uppercase(),
            gen_purp_reg,
        }
    }

    pub fn push_constant(&self, value: i16) -> Vec<String> {
        flatten(vec![set_d_reg_to_constant(value), push_d_reg_to_stack()])
    }

    pub fn push_to_stack(&self, segment: Segment, idx: u16) -> Result<Vec<String>, MemoryError> {
        Ok(flatten(vec![
            match segment {
                Segment::Static => flatten(vec![set_d_reg_to_alias(
                    format!("{}.{}", self.namespace, idx).as_str(),
                    None,
                )]),
                Segment::Temp => {
                    if idx > AVAILABLE_TMP_BLOCKS - 1 {
                        Err(MemoryError::OutOfBounds(idx, segment))?
                    } else {
                        flatten(vec![
                            set_a_reg_to_address(TMP_BASE_ADDR + idx),
                            set_d_reg_to_mem(),
                        ])
                    }
                }
                Segment::Pointer => {
                    if idx > 1 {
                        Err(MemoryError::OutOfBounds(idx, segment))?
                    } else {
                        let segment = if idx == 0 {
                            Segment::This
                        } else {
                            Segment::That
                        };
                        set_d_reg_to_alias(get_segment_alias(&segment), None)
                    }
                }
                segment => flatten(vec![
                    set_a_reg_to_segment_idx(segment, idx),
                    set_d_reg_to_mem(),
                ]),
            },
            push_d_reg_to_stack(),
        ]))
    }

    pub(crate) fn pop_stack_to(
        &self,
        segment: Segment,
        idx: u16,
    ) -> Result<Vec<String>, MemoryError> {
        let tmp = self.gen_purp_reg.borrow_mut().next()?;
        Ok(flatten(vec![match segment {
            Segment::Static => flatten(vec![
                pop_stack_to_d_reg(),
                set_alias(format!("{}.{}", self.namespace, idx).as_str()),
                set_mem_to_d_reg(),
            ]),
            Segment::Temp => {
                if idx > AVAILABLE_TMP_BLOCKS - 1 {
                    Err(MemoryError::OutOfBounds(idx, segment))?
                } else {
                    flatten(vec![
                        pop_stack_to_d_reg(),
                        set_a_reg_to_constant((TMP_BASE_ADDR + idx) as i16),
                        set_mem_to_d_reg(),
                    ])
                }
            }
            Segment::Pointer => {
                if idx > 1 {
                    Err(MemoryError::OutOfBounds(idx, segment))?
                } else {
                    let segment = if idx == 0 {
                        Segment::This
                    } else {
                        Segment::That
                    };
                    flatten(vec![
                        pop_stack_to_d_reg(),
                        set_alias(get_segment_alias(&segment)),
                        set_mem_to_d_reg(),
                    ])
                }
            }
            segment => flatten(vec![
                set_d_reg_to_segment_idx(segment, idx),
                set_alias(tmp.to_string().as_str()),
                set_mem_to_d_reg(),
                pop_stack_to_d_reg(),
                set_mem_at_alias_to_d_reg(tmp.to_string().as_str()),
            ]),
        }]))
    }
}

pub(super) fn set_d_reg_to_segment_idx(segment: Segment, idx: u16) -> Vec<String> {
    let asm = flatten(vec![set_d_reg_to_alias(
        get_segment_alias(&segment),
        Some(idx as i16),
    )]);
    asm
}

pub(super) fn set_a_reg_to_segment_idx(segment: Segment, idx: u16) -> Vec<String> {
    let alias = get_segment_alias(&segment);
    let asm = flatten(vec![if idx == 0 {
        set_a_reg_to_alias(alias)
    } else {
        flatten(vec![
            set_d_reg_to_alias(alias, None),
            if idx == 1 {
                vec![format!("A=D+1")]
            } else {
                flatten(vec![
                    set_a_reg_to_constant(idx as i16),
                    vec![format!("A=D+A")],
                ])
            },
        ])
    }]);
    asm
}

pub(super) fn get_segment_alias(segment: &Segment) -> &str {
    *SEGMENT_MEM_MAP.get(segment).unwrap()
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
        Segment::Temp,
        AVAILABLE_TMP_BLOCKS;
        "temp"
    )]
    #[test_case(
        Segment::Pointer,
        2;
        "pointer"
    )]
    fn it_should_raise_error_when_pushing_out_of_bounds_segment(segment: Segment, idx: u16) {
        let cmd_writer = make_mem_cmd_writer();
        assert!(cmd_writer.push_to_stack(segment, idx).is_err());
    }

    #[test_case(
        Segment::Temp,
        AVAILABLE_TMP_BLOCKS;
        "temp"
    )]
    #[test_case(
        Segment::Pointer,
        2;
        "pointer"
    )]
    fn it_should_raise_error_when_popping_out_of_bounds_segment(segment: Segment, idx: u16) {
        let cmd_writer = make_mem_cmd_writer();
        assert!(cmd_writer.pop_stack_to(segment, idx).is_err());
    }
}
