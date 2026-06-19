//! Keyboard-driven single-line text input.

mod number_input;
mod state;
mod text_input;

pub use number_input::NumberInput;
pub use state::{TextInputAction, TextInputState};
pub use text_input::TextInput;
