use super::{BaseUnit, Unit};
use crate::error::ParseError;
use crate::parse::Segment;
use core::ops::Deref;
//use drstd as std;
use std::boxed::Box;
use std::rc::Rc;
use std::vec::Vec;

#[derive(Default)]
pub struct TargetUnit {
    unit_base: BaseUnit,
    targets: Vec<Rc<dyn Unit>>,
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
        Ok(Rc::new(TargetUnit::default()))
    }

    fn set_attr(&mut self, segement: Segment, attr: &str, val: &str) -> Result<(), ParseError> {
        Ok(())
    }
}
