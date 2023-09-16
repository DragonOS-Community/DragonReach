use drstd::std as std;
use std::vec::Vec;
use std::boxed::Box;

use super::{Unit, BaseUnit};


pub struct TargetUnit {
    unit_base: BaseUnit,
    targets: Vec<Box<dyn Unit>>
}

impl Unit for TargetUnit {
    fn parse(path: &str) -> Self {
        
    }
}