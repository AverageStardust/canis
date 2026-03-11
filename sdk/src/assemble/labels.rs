use std::collections::HashMap;

use crate::{assemble::line_span::Span, instruction::types::Location};

pub enum LabelRegistry<'a> {
    Incomplete,
    Complete(HashMap<&'a str, (Location, Span)>),
}

impl<'a> LabelRegistry<'a> {
    pub fn get_label_location(&self, label: &'a str) -> Option<Location> {
        match self {
            Self::Incomplete => Some(Location::new()),
            Self::Complete(registry) => registry.get(label).map(|(loc, _)| *loc),
        }
    }
}
