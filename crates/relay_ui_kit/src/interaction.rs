use std::rc::Rc;

use gpui::{App, ClickEvent, Window};

pub(crate) type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(crate) type SharedClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(crate) type SharedSelectHandler = Rc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
