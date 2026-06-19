use std::rc::Rc;

use gpui::{App, ClickEvent, Hsla, KeyDownEvent, Window};

pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SharedClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type DismissHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedDismissHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type SharedSelectHandler = Rc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type ActionHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedActionHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type ChangeHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub type KeyCaptureHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static>;
pub type ColorSelectHandler = Box<dyn Fn(&'static str, Hsla, &mut Window, &mut App) + 'static>;
pub type SharedColorSelectHandler = Rc<dyn Fn(&'static str, Hsla, &mut Window, &mut App) + 'static>;
