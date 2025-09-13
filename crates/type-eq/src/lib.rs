//! Type equality assertions.

#![no_std]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(feature = "nightly", feature(specialization))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod match_on_type;
mod type_eq;

pub use crate::{
    match_on_type::{MatchOnType, match_on_type},
    type_eq::{TypeEq, type_eq},
};
