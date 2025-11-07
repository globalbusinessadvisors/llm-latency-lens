//! Core types and timing engine for LLM Latency Lens
//!
//! This crate provides the foundational types and timing infrastructure
//! for high-precision measurement of LLM API latency.

pub mod error;
pub mod timing;
pub mod types;

pub use error::{Error, Result};
pub use timing::{Clock, Timestamp, TimingEngine};
pub use types::*;
