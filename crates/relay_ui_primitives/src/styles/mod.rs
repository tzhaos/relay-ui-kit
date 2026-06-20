//! Theme, tone, spacing, and motion primitives.
//!
//! # Motion types
//!
//! Data types ([`MotionDuration`], [`MotionDirection`], etc.) live in
//! [`crate::contract::motion`] — the single canonical source.
//!
//! The GPUI animation extension trait [`MotionExt`] lives here in
//! [`styles::motion`] because it depends on GPUI animation primitives.

pub mod motion;
pub mod theme;
pub mod tone;

// Only re-export the MotionExt trait, not the contract data types that
// styles::motion imports from contract::motion.  This avoids the dual
// namespace problem where MotionDuration appears under both
// `contract::motion` and `crate::motion`.
pub use motion::MotionExt;
pub use theme::*;
pub use tone::*;
