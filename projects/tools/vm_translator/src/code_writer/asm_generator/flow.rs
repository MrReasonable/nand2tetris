use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{
    flatten, label,
    memory::{get_segment_alias, set_d_reg_to_segment_idx},
    register::{
        set_a_reg_to_alias, set_a_reg_to_constant, set_alias, set_d_reg_to_a_reg,
        set_d_reg_to_alias, set_d_reg_to_mem, set_mem_to_d_reg, CmpVal,
    },
    stack::{pop_stack_to_d_reg, push_d_reg_to_stack, SEGMENT_STACK},
    MemCmdWriter, MemoryError,
};
use crate::{
    code_writer::{
        label_manager::LabelManager,
        reg_mgr::{Reg, RegMgr, RegMgrError},
    },
    parser::{Flow, Segment},
};

#[derive(thiserror::Error, Debug)]
pub enum FlowError {
    #[error("RegMgr: {0}")]
    RegMgr(#[from] RegMgrError),
    #[error("Memory: {0}")]
    Memory(#[from] MemoryError),
}

pub(crate) enum JmpCmd {
    Jeq,
    Jlt,
    Jgt,
    Jmp,
}

impl Display for JmpCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JmpCmd::Jeq => "JEQ",
                JmpCmd::Jlt => "JLT",
                JmpCmd::Jgt => "JGT",
                JmpCmd::Jmp => "JMP",
            }
        )
    }
}

type FlowCmd = Box<dyn Fn(Flow, &mut LabelManager) -> Result<Vec<String>, FlowError>>;

pub(crate) fn flow(gen_purp_reg: Rc<RefCell<RegMgr>>, mem_cmd_writer: Rc<MemCmdWriter>) -> FlowCmd {
    Box::new(move |flow_cmd, label_manager| match flow_cmd {
        Flow::Goto(goto_type, ref l) => match goto_type {
            crate::parser::Goto::Direct => Ok(goto(l)),
            crate::parser::Goto::Conditional => Ok(if_goto(l)),
        },
        Flow::Call(name, args) => Ok(call(&name, args, label_manager)),
        Flow::Return => {
            label_manager.end_function();
            Ok(return_cmd(gen_purp_reg.clone(), mem_cmd_writer.clone())?)
        }
    })
}

fn goto(label: &str) -> Vec<String> {
    flatten(vec![set_alias(label), jmp(JmpCmd::Jmp, CmpVal::Zero)])
}

fn if_goto(label: &str) -> Vec<String> {
    flatten(vec![
        pop_stack_to_d_reg(),
        set_alias(label),
        jmp(JmpCmd::Jgt, CmpVal::D),
        jmp(JmpCmd::Jlt, CmpVal::D),
    ])
}

fn call(name: &str, arg_count: u8, label_manager: &mut LabelManager) -> Vec<String> {
    let ret_label = label_manager.generate_label(format!("{}$ret", name).as_str(), true);
    flatten(vec![
        generate_retun(&ret_label),
        save_local_frame(),
        reset_args_for_call(arg_count),
        set_d_reg_to_alias(SEGMENT_STACK, None),
        set_alias(get_segment_alias(&Segment::Local)),
        set_mem_to_d_reg(),
        goto(name),
        label(&ret_label),
    ])
}

fn generate_retun(label: &str) -> Vec<String> {
    flatten(vec![
        set_alias(label),
        set_d_reg_to_a_reg(),
        push_d_reg_to_stack(),
    ])
}

fn save_local_frame() -> Vec<String> {
    flatten(vec![
        set_d_reg_to_segment_idx(Segment::Local, 0),
        push_d_reg_to_stack(),
        set_d_reg_to_segment_idx(Segment::Argument, 0),
        push_d_reg_to_stack(),
        set_d_reg_to_segment_idx(Segment::This, 0),
        push_d_reg_to_stack(),
        set_d_reg_to_segment_idx(Segment::That, 0),
        push_d_reg_to_stack(),
    ])
}

fn reset_args_for_call(arg_count: u8) -> Vec<String> {
    flatten(vec![
        set_d_reg_to_alias(SEGMENT_STACK, Some(-(5 + arg_count as i16))),
        set_alias(get_segment_alias(&Segment::Argument)),
        set_mem_to_d_reg(),
    ])
}

fn return_cmd(
    gen_purp_reg: Rc<RefCell<RegMgr>>,
    mem_cmd_writer: Rc<MemCmdWriter>,
) -> Result<Vec<String>, FlowError> {
    let lcl = gen_purp_reg.borrow_mut().next()?;
    let ret_add = gen_purp_reg.borrow_mut().next()?;
    Ok(flatten(vec![
        set_d_reg_to_segment_idx(Segment::Local, 0),
        set_alias(&lcl.to_string()),
        set_mem_to_d_reg(),
        set_a_reg_to_constant(5),
        vec!["A=D-A".to_string()],
        set_d_reg_to_mem(),
        set_alias(&ret_add.to_string()),
        set_mem_to_d_reg(),
        mem_cmd_writer.pop_stack_to(Segment::Argument, 0)?,
        set_d_reg_to_segment_idx(Segment::Argument, 1),
        set_alias(SEGMENT_STACK),
        set_mem_to_d_reg(),
        set_segment_addr(&lcl, 1, Segment::That),
        set_segment_addr(&lcl, 2, Segment::This),
        set_segment_addr(&lcl, 3, Segment::Argument),
        set_segment_addr(&lcl, 4, Segment::Local),
        set_a_reg_to_alias(&ret_add.to_string()),
        jmp(JmpCmd::Jmp, CmpVal::Zero),
    ]))
}

fn set_segment_addr(reg: &Reg, steps_back: u8, segment: Segment) -> Vec<String> {
    flatten(vec![
        if steps_back == 1 {
            flatten(vec![set_alias(&reg.to_string()), vec!["A=M-1".to_string()]])
        } else {
            flatten(vec![
                set_d_reg_to_alias(&reg.to_string(), None),
                set_a_reg_to_constant(steps_back as i16),
                vec!["A=D-A".to_string()],
            ])
        },
        set_d_reg_to_mem(),
        set_alias(get_segment_alias(&segment)),
        set_mem_to_d_reg(),
    ])
}

pub(super) fn jmp(jmp_cmd: JmpCmd, cmp_val: CmpVal) -> Vec<String> {
    vec![format!("{};{}", cmp_val, jmp_cmd)]
}
