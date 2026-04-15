#![doc = include_str!("../README.md")]

#[macro_use]
mod core;

pub mod prelude;

pub use crate::core::{CExpr, EExpr, LazyCExpr, LazyEExpr};
