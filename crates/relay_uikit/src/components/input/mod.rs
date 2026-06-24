//! Text and numeric input primitives backed by real desktop editing behavior.
//!
//! This family is responsible for the hardest part of product UI correctness:
//! text entry, caret movement, selection, composition, and numeric editing.
//!
//! The public surfaces are split by role:
//!
//! - [`TextInput`] for single-line text entry;
//! - [`SearchField`] for search-shaped text entry with search affordance;
//! - [`TextArea`] for multiline editing;
//! - [`NumberInput`] for structured numeric entry with product-facing layout
//!   options;
//! - [`TextInputState`] for host-owned editing state where the host needs to
//!   retain cursor, selection, or composition state explicitly.
//!
//! Internally this family is wired through GPUI's platform text input path
//! rather than pretending desktop text editing can be modeled as raw key events
//! alone. That is the foundation for IME correctness, UTF-16 range conversion,
//! and real composition behavior.
//!
//! Use this module when the host needs product-grade editing behavior. Reach for
//! [`crate::patterns::InputComposer`] only when you need a larger composed input
//! surface with surrounding chrome and actions.

mod number_input;
mod platform_input;
mod search_field;
mod state;
mod text_area;
mod text_input;

pub use number_input::{NumberInput, NumberInputLayout};
pub use search_field::SearchField;
pub use state::TextInputState;
pub use text_area::TextArea;
pub use text_input::TextInput;
