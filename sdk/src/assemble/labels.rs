use std::collections::HashMap;

use crate::{assemble::line_span::Span, instruction::types::Location};

pub enum LabelRegistry<'a> {
    Incomplete,
    Complete(HashMap<&'a str, (Location, Span)>),
}

pub enum LabelRegistryResult<T> {
    Found(T),
    NotFound,
    Incomplete,
}

impl<T> From<Option<T>> for LabelRegistryResult<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Found(value),
            None => Self::NotFound,
        }
    }
}

impl<'a> LabelRegistry<'a> {
    pub fn get_label(&self, label: &'a str) -> LabelRegistryResult<(Location, Span)> {
        match self {
            Self::Incomplete => LabelRegistryResult::Incomplete,
            Self::Complete(registry) => registry.get(label).map(|(loc, span)| (*loc, *span)).into(),
        }
    }
}
