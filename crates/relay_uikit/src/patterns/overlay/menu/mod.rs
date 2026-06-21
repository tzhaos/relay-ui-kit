mod item;

use gpui::{
    App, Bounds, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder,
    px,
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
}

#[derive(Clone, Copy)]
struct MenuSnapshot {
    open_submenu: Option<usize>,
    submenu_offset: Option<Pixels>,
}

impl MenuState {
    fn snapshot(&self) -> MenuSnapshot {
        MenuSnapshot {
            open_submenu: self.open_submenu,
            submenu_offset: self.submenu_offset(),
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
}

impl Menu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            min_width: 180.0,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id.clone();
        let state =
            window.use_keyed_state((id.clone(), "menu-state"), cx, |_, _| MenuState::default());

        // Reset stale open_submenu if it points beyond the current items.
        // This can happen when the menu is re-rendered with fewer items
        // than before (e.g. dropdown reopened with different content).
        let item_count = self.items.len();
        {
            let needs_reset = state.read(cx).open_submenu.is_some_and(|idx| idx >= item_count);
            if needs_reset {
                state.update(cx, |state, cx| {
                    if state.close_submenu() {
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
            .min_w(px(self.min_width))
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
            .child(menu_bounds_measure);
        let mut open_submenu: Option<(usize, Vec<MenuItem>, Pixels)> = None;

        for (index, item) in self.items.into_iter().enumerate() {
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

            let fg = if danger { theme.danger } else { theme.text };
            let icon_color = if danger {
                theme.danger
            } else {
                theme.text_muted
            };
            let has_submenu = submenu || !submenu_items.is_empty();
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
                .text_color(fg)
                .when(disabled, |this| this.opacity(DISABLED_OPACITY))
                .when(submenu_visible, |this| this.bg(theme.hover))
                .when(!disabled, |this| {
                    this.cursor_pointer().hover(move |s| s.bg(theme.hover))
                })
                .on_hover(move |hovered, _window, cx| {
                    if disabled || !*hovered {
                        return;
                    }

                    state_for_hover.update(cx, |state, cx| {
                        let changed = if has_submenu {
                            state.open_submenu(index)
                        } else {
                            state.close_submenu()
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
                            if state.open_submenu(index) {
                                cx.notify();
                            }
                        });
                        cx.stop_propagation();
                    })
                })
                .when_some(
                    on_click.filter(|_| !disabled && !has_submenu),
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
}
