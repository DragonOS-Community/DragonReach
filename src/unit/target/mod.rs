use super::{BaseUnit, Unit};
use crate::error::parse_error::ParseError;
use crate::parse::Segment;
use crate::parse::parse_target::TargetParser;
use core::ops::Deref;
use std::sync::Arc;
use cfg_if::cfg_if;

cfg_if!{
    if #[cfg(target_os = "dragonos")]{
        use drstd as std;
        use std::rc::Rc;
        use std::vec::Vec;
    }else{
        use std::rc::Rc;
        use std::vec::Vec;
    }
}

#[derive(Debug,Clone,Default)]
pub struct TargetUnit {
    unit_base: BaseUnit,
    //targets: Vec<Rc<dyn Unit>>,
}

impl Unit for TargetUnit {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn from_path(path: &str) -> Result<Arc<Self>, ParseError>
    where
        Self: Sized,
    {
        return TargetParser::parse(path);
    }

    fn set_attr(&mut self, segement: Segment, attr: &str, val: &str) -> Result<(), ParseError> {
        Ok(())
    }

    fn set_unit_base(&mut self, base: BaseUnit) {
        self.unit_base = base;
    }

    fn unit_type(&self) -> super::UnitType {
        return self.unit_base.unit_type;
    }

    fn unit_base(&self) -> &BaseUnit {
        return &self.unit_base;
    }

    fn unit_id(&self) -> usize {
        return self.unit_base.unit_id;
    }

    fn run(&self) -> Result<(),crate::error::runtime_error::RuntimeError> {
        Ok(())
    }

    fn mut_unit_base(&mut self) -> &mut BaseUnit {
        return &mut self.unit_base;
    }
}

unsafe impl Sync for TargetUnit {}

unsafe impl Send for TargetUnit {}
