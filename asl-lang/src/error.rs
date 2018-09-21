use debug_info::{Span, SrcByteRange};
use specs::prelude::*;

#[derive(Debug)]
pub struct RangeError {
    pub message: String,
    pub range: Option<SrcByteRange>,
}

pub type RangeResult<T> = ::std::result::Result<T, RangeError>;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub span: Option<Span>,
}

impl RangeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            range: None,
        }
    }
    pub fn spanned(self, src: &str) -> Error {
        Error {
            message: self.message,
            span: self.range.map(|r| r.to_span(src)),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub trait ResultExt {
    fn with_entity_range(self, entity: Entity, ranges: &ReadStorage<SrcByteRange>) -> Self;
}

impl<T> ResultExt for RangeResult<T> {
    fn with_entity_range(self, entity: Entity, ranges: &ReadStorage<SrcByteRange>) -> Self {
        match self {
            Ok(o) => Ok(o),
            Err(mut e) => {
                if let Some(range) = ranges.get(entity) {
                    e.range = Some(*range);
                }
                Err(e)
            }
        }
    }
}
