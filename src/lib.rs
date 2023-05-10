#![no_std]
#![feature(const_trait_impl)]
#![feature(try_blocks)]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

//! A G-code parser designed for embedded environments.
//!
//! ## Feature flags
#![doc = document_features::document_features!()]

mod block;
mod decimal;
mod sign;
mod significand;

pub use crate::block::{BlockBuilder, BlockParser, Error, ErrorKind};
pub use crate::decimal::Decimal;
pub use crate::significand::Significand;
