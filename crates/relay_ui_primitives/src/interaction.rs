use std::rc::Rc;

use gpui::{App, ClickEvent, Window};

pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SharedClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SharedSelectHandler = Rc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
