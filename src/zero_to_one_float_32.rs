use std::ops::RangeInclusive;
use typed_floats::{tf32::PositiveFinite, InvalidNumber};

#[derive(Debug)]
pub enum InvalidZeroToOne {
    GreaterThanOne,
    PositiveFinite(InvalidNumber),
}

impl From<InvalidNumber> for InvalidZeroToOne {
    fn from(value: InvalidNumber) -> Self {
        Self::PositiveFinite(value)
    }
}

/// A number inclusively between 0.0 and 1.0
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct ZeroToOneFloat32(PositiveFinite);

impl ZeroToOneFloat32 {
    pub fn new(inner: f32) -> Result<Self, InvalidZeroToOne> {
        if inner > 1.0 {
            Err(InvalidZeroToOne::GreaterThanOne)
        } else {
            PositiveFinite::new(inner)
                .map_err(InvalidZeroToOne::PositiveFinite)
                .map(Self)
        }
    }

    pub fn range_inclusive() -> RangeInclusive<f32> {
        0.0..=1.0
    }
}

impl From<ZeroToOneFloat32> for f32 {
    fn from(value: ZeroToOneFloat32) -> Self {
        value.0.into()
    }
}
