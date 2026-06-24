mod item;

use gpui::{
    App, Bounds, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, canvas,
    div, prelude::FluentBuilder, px,
};

pub use item::MenuItem;

use crate::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY, radius, space},
};

const SUBMENU_MIN_WIDTH: f32 = 180.0;

#[derive(Default)]
struct MenuState {
    open_submenu: Option<usize>,
    menu_bounds: Option<Bounds<Pixels>>,
    trigger_bounds: Option<(usize, Bounds<Pixels>)>,
    active_index: Option<usize>,
}

#[derive(Clone, Copy)]
struct MenuSnapshot {
    open_submenu: Option<usize>,
    submenu_offset: Option<Pixels>,
    active_index: Option<usize>,
}

#[derive(Clone)]
struct MenuKeyboardItem {
    interactive: bool,
    checked: bool,
    has_submenu: bool,
    on_click: Option<crate::interaction::SharedClickHandler>,
}

#[derive(Clone, Copy)]
enum MenuMove {
    First,
    Last,
    Next,
    Previous,
}

impl MenuState {
    fn snapshot(&self) -> MenuSnapshot {
        MenuSnapshot {
            open_submenu: self.open_submenu,
            submenu_offset: self.submenu_offset(),
            active_index: self.active_index,
        }
    }

    fn open_submenu(&mut self, index: usize) -> bool {
        if self.open_submenu == Some(index) {
            return false;
        }

        self.open_submenu = Some(index);
        self.trigger_bounds = None;
        true
    }

    fn close_submenu(&mut self) -> bool {
        if self.open_submenu.is_none() {
            return false;
        }

        self.open_submenu = None;
        self.trigger_bounds = None;
        true
    }

    fn set_menu_bounds(&mut self, bounds: Bounds<Pixels>) -> bool {
        if self.menu_bounds == Some(bounds) {
            return false;
        }

        self.menu_bounds = Some(bounds);
        true
    }

    fn set_trigger_bounds(&mut self, index: usize, bounds: Bounds<Pixels>) -> bool {
        let next = Some((index, bounds));
        if self.trigger_bounds == next {
            return false;
        }

        self.trigger_bounds = next;
        true
    }

    fn set_active_index(&mut self, index: Option<usize>) -> bool {
        if self.active_index == index {
            return false;
        }

        self.active_index = index;
        true
    }

    fn submenu_offset(&self) -> Option<Pixels> {
        let open_index = self.open_submenu?;
        let menu_bounds = self.menu_bounds?;
        let (trigger_index, trigger_bounds) = self.trigger_bounds?;

        (trigger_index == open_index).then_some(trigger_bounds.origin.y - menu_bounds.origin.y)
    }
}

/// A floating menu panel.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    items: Vec<MenuItem>,
    min_width: f32,
    focus_handle: Option<FocusHandle>,
}

impl Menu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            min_width: 180.0,
            focus_handle: None,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }
}

fn menu_item_is_interactive(item: &MenuKeyboardItem) -> bool {
    item.interactive
}

fn preferred_active_index(items: &[MenuKeyboardItem]) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .find(|(_, item)| menu_item_is_interactive(item) && item.checked)
        .or_else(|| {
            items
                .iter()
                .enumerate()
                .find(|(_, item)| menu_item_is_interactive(item))
        })
        .map(|(index, _)| index)
}

fn normalized_active_index(
    items: &[MenuKeyboardItem],
    active_index: Option<usize>,
) -> Option<usize> {
    active_index
        .filter(|&index| items.get(index).is_some_and(menu_item_is_interactive))
        .or_else(|| preferred_active_index(items))
}

fn moved_active_index(
    items: &[MenuKeyboardItem],
    active_index: Option<usize>,
    movement: MenuMove,
) -> Option<usize> {
    let interactive_indices = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| menu_item_is_interactive(item).then_some(index))
        .collect::<Vec<_>>();
    if interactive_indices.is_empty() {
        return None;
    }

    match movement {
        MenuMove::First => interactive_indices.first().copied(),
        MenuMove::Last => interactive_indices.last().copied(),
        MenuMove::Next | MenuMove::Previous => {
            let fallback = match movement {
                MenuMove::Next => 0,
                MenuMove::Previous => interactive_indices.len().saturating_sub(1),
                MenuMove::First | MenuMove::Last => unreachable!(),
            };
            let current = normalized_active_index(items, active_index);
            let current_pos = current
                .and_then(|index| {
                    interactive_indices
                        .iter()
                        .position(|candidate| *candidate == index)
                })
                .unwrap_or(fallback);
            let next_pos = match movement {
                MenuMove::Next => (current_pos + 1) % interactive_indices.len(),
                MenuMove::Previous => {
                    if current_pos == 0 {
                        interactive_indices.len().saturating_sub(1)
                    } else {
                        current_pos - 1
                    }
                }
                MenuMove::First | MenuMove::Last => unreachable!(),
            };
            interactive_indices.get(next_pos).copied()
        }
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            items,
            min_width,
            focus_handle,
        } = self;
        let state =
            window.use_keyed_state((id.clone(), "menu-state"), cx, |_, _| MenuState::default());
        let focus_handle = focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "focus-handle"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let keyboard_items = std::rc::Rc::new(
            items
                .iter()
                .map(|item| {
                    let has_submenu = item.submenu || !item.submenu_items.is_empty();
                    MenuKeyboardItem {
                        interactive: !item.separator
                            && !item.header
                            && !item.disabled
                            && (has_submenu || item.on_click.is_some()),
                        checked: item.checked,
                        has_submenu,
                        on_click: item.on_click.clone(),
                    }
                })
                .collect::<Vec<_>>(),
        );

        // Reset stale open_submenu if it points beyond the current items.
        // This can happen when the menu is re-rendered with fewer items
        // than before (e.g. dropdown reopened with different content).
        let item_count = items.len();
        let current_state = state.read(cx).snapshot();
        let normalized_active =
            normalized_active_index(&keyboard_items, current_state.active_index);
        {
            let needs_reset = current_state
                .open_submenu
                .is_some_and(|idx| idx >= item_count);
            let needs_active_sync = current_state.active_index != normalized_active;
            if needs_reset || needs_active_sync {
                state.update(cx, |state, cx| {
                    let mut changed = false;
                    if needs_reset {
                        changed |= state.close_submenu();
                    }
                    changed |= state.set_active_index(normalized_active);
                    if changed {
                        cx.notify();
                    }
                });
            }
        }

        let snapshot = state.read(cx).snapshot();

        let menu_bounds_state = state.clone();
        let menu_bounds_measure = canvas(
            move |bounds, _window, cx| {
                menu_bounds_state.update(cx, |state, cx| {
                    if state.set_menu_bounds(bounds) {
                        cx.notify();
                    }
                });
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        let mut panel = div()
            .id((id.clone(), "panel"))
            .relative()
            .min_w(px(min_width))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(BORDER_WIDTH))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude()
            .role(Role::Menu)
            .tab_index(0)
            .track_focus(&focus_handle)
            .on_key_down({
                let state = state.clone();
                let keyboard_items = keyboard_items.clone();
                move |event: &KeyDownEvent, window, cx| {
                    let snapshot = state.read(cx).snapshot();
                    let active_index =
                        normalized_active_index(&keyboard_items, snapshot.active_index);
                    let set_active = |next_index: Option<usize>, cx: &mut App| {
                        state.update(cx, |state, cx| {
                            let mut changed = state.set_active_index(next_index);
                            if state.open_submenu != next_index {
                                changed |= state.close_submenu();
                            }
                            if changed {
                                cx.notify();
                            }
                        });
                    };
                    match event.keystroke.key.as_str() {
                        "down" => {
                            set_active(
                                moved_active_index(&keyboard_items, active_index, MenuMove::Next),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "up" => {
                            set_active(
                                moved_active_index(
                                    &keyboard_items,
                                    active_index,
                                    MenuMove::Previous,
                                ),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "home" => {
                            set_active(
                                moved_active_index(&keyboard_items, active_index, MenuMove::First),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "end" => {
                            set_active(
                                moved_active_index(&keyboard_items, active_index, MenuMove::Last),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "right" => {
                            if let Some(index) = active_index
                                && keyboard_items
                                    .get(index)
                                    .is_some_and(|item| item.has_submenu)
                            {
                                state.update(cx, |state, cx| {
                                    let mut changed = state.set_active_index(Some(index));
                                    changed |= state.open_submenu(index);
                                    if changed {
                                        cx.notify();
                                    }
                                });
                                cx.stop_propagation();
                            }
                        }
                        "left" => {
                            state.update(cx, |state, cx| {
                                if state.close_submenu() {
                                    cx.notify();
                                }
                            });
                            cx.stop_propagation();
                        }
                        "enter" | " " => {
                            if let Some(index) = active_index
                                && let Some(item) = keyboard_items.get(index)
                            {
                                if item.has_submenu {
                                    state.update(cx, |state, cx| {
                                        let mut changed = state.set_active_index(Some(index));
                                        changed |= state.open_submenu(index);
                                        if changed {
                                            cx.notify();
                                        }
                                    });
                                } else if let Some(handler) = &item.on_click {
                                    handler(&gpui::ClickEvent::default(), window, cx);
                                }
                                cx.stop_propagation();
                            }
                        }
                        "escape" if snapshot.open_submenu.is_some() => {
                            state.update(cx, |state, cx| {
                                if state.close_submenu() {
                                    cx.notify();
                                }
                            });
                            cx.stop_propagation();
                        }
                        "escape" => {}
                        _ => {}
                    }
                }
            })
            .child(menu_bounds_measure);
        let mut open_submenu: Option<(usize, Vec<MenuItem>, Pixels)> = None;

        for (index, item) in items.into_iter().enumerate() {
            let MenuItem {
                label,
                detail,
                icon,
                trailing,
                checked,
                danger,
                disabled,
                separator,
                header,
                submenu,
                submenu_items,
                on_click,
            } = item;

            if separator {
                panel = panel.child(
                    div()
                        .my(px(space::XS))
                        .h(px(BORDER_WIDTH))
                        .w_full()
                        .bg(theme.border),
                );
                continue;
            }

            if header {
                panel = panel.child(menu_header(label, theme.text_muted));
                continue;
            }

            let interactive = keyboard_items
                .get(index)
                .is_some_and(menu_item_is_interactive);
            let fg = if danger { theme.danger } else { theme.text };
            let icon_color = if danger {
                theme.danger
            } else {
                theme.text_muted
            };
            let has_submenu = submenu || !submenu_items.is_empty();
            let row_active = snapshot.active_index == Some(index);
            let submenu_visible = snapshot.open_submenu == Some(index) && !submenu_items.is_empty();
            let state_for_hover = state.clone();
            let state_for_click = state.clone();
            let state_for_measure = state.clone();
            let mut row_content = div()
                .id(("menu-item", index))
                .relative()
                .min_h(px(if detail.is_some() { 38.0 } else { 28.0 }))
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .gap_2()
                .rounded(px(radius::MD))
                .text_sm()
                .role(if checked {
                    Role::MenuItemCheckBox
                } else {
                    Role::MenuItem
                })
                .aria_label(label.clone())
                .text_color(fg)
                .when(disabled, |this| this.opacity(DISABLED_OPACITY))
                .when(row_active || submenu_visible, |this| this.bg(theme.hover))
                .when(interactive, |this| {
                    this.cursor_pointer().hover(move |s| s.bg(theme.hover))
                })
                .on_hover(move |hovered, _window, cx| {
                    if !interactive || !*hovered {
                        return;
                    }

                    state_for_hover.update(cx, |state, cx| {
                        let changed = state.set_active_index(Some(index));
                        let changed = if has_submenu {
                            changed | state.open_submenu(index)
                        } else {
                            changed | state.close_submenu()
                        };
                        if changed {
                            cx.notify();
                        }
                    });
                })
                .child(menu_leading_icon(checked, icon, icon_color, theme.accent))
                .child(menu_label(label, detail, theme.text_muted))
                .when_some(trailing, |this, trailing| this.child(trailing))
                .when(has_submenu, |this| {
                    this.child(
                        Icon::new(IconName::ChevronRight)
                            .size(IconSize::XSmall)
                            .color(theme.text_muted),
                    )
                })
                .when(has_submenu && !disabled, |this| {
                    this.on_click(move |_event, _window, cx| {
                        state_for_click.update(cx, |state, cx| {
                            let mut changed = state.set_active_index(Some(index));
                            changed |= state.open_submenu(index);
                            if changed {
                                cx.notify();
                            }
                        });
                        cx.stop_propagation();
                    })
                })
                .when_some(
                    on_click.filter(|_| interactive && !has_submenu),
                    |this, handler| {
                        this.on_click(move |event, window, cx| {
                            handler(event, window, cx);
                            cx.stop_propagation();
                        })
                    },
                );

            if submenu_visible {
                row_content = row_content.child(
                    canvas(
                        move |bounds, _window, cx| {
                            state_for_measure.update(cx, |state, cx| {
                                if state.set_trigger_bounds(index, bounds) {
                                    cx.notify();
                                }
                            });
                        },
                        |_bounds, (), _window, _cx| {},
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                );
            }

            if submenu_visible && let Some(offset) = snapshot.submenu_offset {
                open_submenu = Some((index, submenu_items, offset));
            }

            panel = panel.child(div().relative().child(row_content));
        }

        div()
            .id(id)
            .relative()
            .flex()
            .items_start()
            .child(panel)
            .when_some(open_submenu, |this, (index, submenu_items, offset)| {
                this.child(div().mt(offset).ml(px(-BORDER_WIDTH)).child(
                    Menu::new(("submenu", index), submenu_items).min_width(SUBMENU_MIN_WIDTH),
                ))
            })
            .motion_slide_in(MotionDirection::FromTop, true)
    }
}

fn menu_leading_icon(
    checked: bool,
    icon: Option<IconName>,
    icon_color: gpui::Hsla,
    accent: gpui::Hsla,
) -> impl IntoElement {
    div()
        .w(px(16.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .when(checked, |this| {
            this.child(
                Icon::new(IconName::Check)
                    .size(IconSize::Small)
                    .color(accent),
            )
        })
        .when(!checked, |this| {
            this.when_some(icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small).color(icon_color))
            })
        })
}

fn menu_label(label: String, detail: Option<String>, detail_color: gpui::Hsla) -> impl IntoElement {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(BORDER_WIDTH))
        .child(
            div()
                .truncate()
                .font_weight(FontWeight::MEDIUM)
                .child(label),
        )
        .when_some(detail, |this, detail| {
            this.child(
                div()
                    .truncate()
                    .text_size(px(11.0))
                    .text_color(detail_color)
                    .child(detail),
            )
        })
}

fn menu_header(label: String, color: gpui::Hsla) -> impl IntoElement {
    div()
        .min_h(px(24.0))
        .px_2()
        .flex()
        .items_center()
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(label)
}

#[cfg(test)]
mod tests {
    use gpui::{bounds, point, size};

    use super::*;

    #[test]
    fn menu_state_has_no_open_submenu_by_default() {
        let state = MenuState::default();

        assert_eq!(state.snapshot().open_submenu, None);
        assert_eq!(state.snapshot().active_index, None);
    }

    #[test]
    fn menu_state_switching_submenu_clears_stale_trigger_bounds() {
        let mut state = MenuState::default();
        state.open_submenu(1);
        state.set_trigger_bounds(1, bounds(point(px(0.0), px(24.0)), size(px(1.0), px(1.0))));

        state.open_submenu(2);

        assert!(state.snapshot().submenu_offset.is_none());
    }

    #[test]
    fn menu_state_computes_submenu_offset_from_trigger_bounds() {
        let mut state = MenuState::default();
        state.open_submenu(1);
        state.set_menu_bounds(bounds(
            point(px(20.0), px(100.0)),
            size(px(200.0), px(180.0)),
        ));
        state.set_trigger_bounds(
            1,
            bounds(point(px(20.0), px(148.0)), size(px(200.0), px(28.0))),
        );

        assert_eq!(state.snapshot().submenu_offset, Some(px(48.0)));
    }

    #[test]
    fn menu_state_close_submenu_clears_trigger_bounds() {
        let mut state = MenuState::default();
        state.open_submenu(1);
        state.set_trigger_bounds(1, bounds(point(px(0.0), px(24.0)), size(px(1.0), px(1.0))));

        state.close_submenu();

        assert_eq!(state.snapshot().open_submenu, None);
        assert!(state.snapshot().submenu_offset.is_none());
    }

    #[test]
    fn preferred_active_index_prefers_checked_interactive_item() {
        let items = vec![
            MenuKeyboardItem {
                interactive: false,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
            MenuKeyboardItem {
                interactive: true,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
            MenuKeyboardItem {
                interactive: true,
                checked: true,
                has_submenu: false,
                on_click: None,
            },
        ];

        assert_eq!(preferred_active_index(&items), Some(2));
    }

    #[test]
    fn moved_active_index_skips_noninteractive_items_and_wraps() {
        let items = vec![
            MenuKeyboardItem {
                interactive: false,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
            MenuKeyboardItem {
                interactive: true,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
            MenuKeyboardItem {
                interactive: false,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
            MenuKeyboardItem {
                interactive: true,
                checked: false,
                has_submenu: false,
                on_click: None,
            },
        ];

        assert_eq!(moved_active_index(&items, None, MenuMove::First), Some(1));
        assert_eq!(moved_active_index(&items, Some(1), MenuMove::Next), Some(3));
        assert_eq!(moved_active_index(&items, Some(3), MenuMove::Next), Some(1));
        assert_eq!(
            moved_active_index(&items, Some(1), MenuMove::Previous),
            Some(3)
        );
        assert_eq!(moved_active_index(&items, None, MenuMove::Last), Some(3));
    }
}
