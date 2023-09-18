use super::{BaseUnit, Unit};
use crate::error::ParseError;
use crate::parse::Segment;
use crate::parse::parse_target::TargetParser;
use core::ops::Deref;
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

#[derive(Default)]
pub struct TargetUnit {
    unit_base: BaseUnit,
    //targets: Vec<Rc<dyn Unit>>,
}

impl Deref for TargetUnit {
    type Target = TargetUnit;
    fn deref(&self) -> &Self::Target {
        &self
    }
}

impl Unit for TargetUnit {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn from_path(path: &str) -> Result<Rc<Self>, ParseError>
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
}
