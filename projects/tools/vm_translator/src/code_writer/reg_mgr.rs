use std::{fmt::Display, rc::Rc};

#[derive(thiserror::Error, Debug)]
pub enum RegMgrError {
    #[error("Invalid range: {0}")]
    InvalidRange(String),
    #[error("No temp space available")]
    NoFreeTmpSpace,
}

pub(crate) struct RegMgr {
    registers: Vec<Rc<String>>,
}

#[derive(Clone)]
pub(crate) struct Reg(Rc<String>);

impl RegMgr {
    pub(super) fn new(start: u8, end: u8) -> Result<Self, RegMgrError> {
        if start > end {
            Err(RegMgrError::InvalidRange(format!("{}:{}", start, end)))
        } else {
            Ok(Self {
                registers: (start..=end).map(|i| Rc::new(format!("R{}", i))).collect(),
            })
        }
    }

    pub(super) fn next(&mut self) -> Result<Reg, RegMgrError> {
        self.registers
            .iter()
            .find(|i| Rc::strong_count(i) < 2)
            .map(|i| Reg(i.clone()))
            .ok_or(RegMgrError::NoFreeTmpSpace)
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_generates_error_for_negative_length() {
        let mgr = RegMgr::new(2, 1);
        assert!(mgr.is_err())
    }

    #[test]
    fn it_generates_register_for_one_register_item() {
        let mut mgr = RegMgr::new(0, 1).unwrap();
        let reg = mgr.next().unwrap();
        assert_eq!(reg.0.to_string(), "R0".to_owned());
    }

    #[test]
    fn it_generates_register_for_entire_range() {
        let mut mgr = RegMgr::new(0, 9).unwrap();
        let mut regs = Vec::new();
        for i in 0..=9 {
            let next = mgr.next();
            assert!(next.is_ok());
            let next = next.unwrap();
            assert_eq!(next.to_string(), format!("R{}", i));
            regs.push(next);
        }
    }

    #[test]
    fn it_reuses_released_regs() {
        let mut mgr = RegMgr::new(0, 9).unwrap();
        for _ in 0..=9 {
            let next = mgr.next();
            assert!(next.is_ok());
            let next = next.unwrap();
            assert_eq!(next.to_string(), "R0".to_string());
        }
    }

    #[test]
    fn it_raises_an_error_when_no_more_regs_available() {
        let mut mgr = RegMgr::new(0, 9).unwrap();
        let mut regs = Vec::new();
        for i in 0..=10 {
            let next = mgr.next();
            if i > 9 {
                assert!(next.is_err())
            } else {
                regs.push(next);
            }
        }
    }
}
