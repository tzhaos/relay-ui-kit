use std::hash::Hash;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{SelectionSource, SharedActionHandler},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, radius, space},
};
use gpui::{
    App, ClickEvent, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use super::picker_types::{PickerAction, PickerOption};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickerPanelEntry {
    Item(usize),
    Action(usize),
}

#[derive(Default)]
struct PickerPanelState {
    active_entry: Option<PickerPanelEntry>,
}

#[derive(Clone, Copy)]
struct PickerKeyboardEntry {
    target: PickerPanelEntry,
    selected: bool,
}

#[derive(Clone, Copy)]
enum PickerMove {
    First,
    Last,
    Next,
    Previous,
}

pub(super) struct BranchPickerPanelProps<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    pub(super) id: ElementId,
    pub(super) focus_handle: FocusHandle,
    pub(super) selected_key: Option<K>,
    pub(super) items: Vec<PickerOption<K>>,
    pub(super) actions: Vec<PickerAction>,
    pub(super) selection: Option<SelectionSource<K>>,
    pub(super) select_handler: Option<SharedActionHandler<K>>,
    pub(super) action_handler: Option<SharedActionHandler<String>>,
}

pub(super) fn branch_picker_panel<K>(props: BranchPickerPanelProps<K>) -> impl IntoElement
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    let BranchPickerPanelProps {
        id,
        focus_handle,
        selected_key,
        items,
        actions,
        selection,
        select_handler,
        action_handler,
    } = props;

    PickerPanel {
        id,
        focus_handle,
        selected_key,
        items,
        actions,
        selection,
        select_handler,
        action_handler,
    }
}

#[derive(IntoElement)]
struct PickerPanel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    focus_handle: FocusHandle,
    selected_key: Option<K>,
    items: Vec<PickerOption<K>>,
    actions: Vec<PickerAction>,
    selection: Option<SelectionSource<K>>,
    select_handler: Option<SharedActionHandler<K>>,
    action_handler: Option<SharedActionHandler<String>>,
}

fn preferred_picker_entry(entries: &[PickerKeyboardEntry]) -> Option<PickerPanelEntry> {
    entries
        .iter()
        .find(|entry| entry.selected)
        .or_else(|| entries.first())
        .map(|entry| entry.target)
}

fn normalized_picker_entry(
    entries: &[PickerKeyboardEntry],
    active_entry: Option<PickerPanelEntry>,
) -> Option<PickerPanelEntry> {
    active_entry
        .filter(|active_entry| entries.iter().any(|entry| entry.target == *active_entry))
        .or_else(|| preferred_picker_entry(entries))
}

fn moved_picker_entry(
    entries: &[PickerKeyboardEntry],
    active_entry: Option<PickerPanelEntry>,
    movement: PickerMove,
) -> Option<PickerPanelEntry> {
    if entries.is_empty() {
        return None;
    }

    match movement {
        PickerMove::First => entries.first().map(|entry| entry.target),
        PickerMove::Last => entries.last().map(|entry| entry.target),
        PickerMove::Next | PickerMove::Previous => {
            let fallback = match movement {
                PickerMove::Next => 0,
                PickerMove::Previous => entries.len().saturating_sub(1),
                PickerMove::First | PickerMove::Last => unreachable!(),
            };
            let current = normalized_picker_entry(entries, active_entry);
            let current_pos = current
                .and_then(|target| entries.iter().position(|entry| entry.target == target))
                .unwrap_or(fallback);
            let next_pos = match movement {
                PickerMove::Next => (current_pos + 1) % entries.len(),
                PickerMove::Previous => {
                    if current_pos == 0 {
                        entries.len().saturating_sub(1)
                    } else {
                        current_pos - 1
                    }
                }
                PickerMove::First | PickerMove::Last => unreachable!(),
            };
            entries.get(next_pos).map(|entry| entry.target)
        }
    }
}

impl<K> RenderOnce for PickerPanel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            focus_handle,
            selected_key,
            items,
            actions,
            selection,
            select_handler,
            action_handler,
        } = self;
        let selected_key = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(selected_key);
        let item_clickable = selection.is_some() || select_handler.is_some();
        let action_clickable = action_handler.is_some();
        let item_keys = std::rc::Rc::new(
            items
                .iter()
                .map(|item| item.key.clone())
                .collect::<Vec<_>>(),
        );
        let action_keys = std::rc::Rc::new(
            actions
                .iter()
                .map(|action| action.key.clone())
                .collect::<Vec<_>>(),
        );
        let keyboard_entries = std::rc::Rc::new(
            items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    item_clickable.then_some(PickerKeyboardEntry {
                        target: PickerPanelEntry::Item(index),
                        selected: selected_key
                            .as_ref()
                            .is_some_and(|selected_key| item.key == *selected_key),
                    })
                })
                .chain(actions.iter().enumerate().filter_map(|(index, _)| {
                    action_clickable.then_some(PickerKeyboardEntry {
                        target: PickerPanelEntry::Action(index),
                        selected: false,
                    })
                }))
                .collect::<Vec<_>>(),
        );
        let state = window.use_keyed_state((id.clone(), "panel-state"), cx, |_, _| {
            PickerPanelState::default()
        });
        let active_entry = normalized_picker_entry(&keyboard_entries, state.read(cx).active_entry);
        if state.read(cx).active_entry != active_entry {
            state.update(cx, |state, cx| {
                state.active_entry = active_entry;
                cx.notify();
            });
        }
        let mut panel = div()
            .id((id, "branch-picker-panel"))
            .w(px(320.0))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(space::XXS))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude()
            .tab_index(0)
            .track_focus(&focus_handle)
            .on_key_down({
                let state = state.clone();
                let keyboard_entries = keyboard_entries.clone();
                let selection = selection.clone();
                let select_handler = select_handler.clone();
                let action_handler = action_handler.clone();
                let item_keys = item_keys.clone();
                let action_keys = action_keys.clone();
                move |event: &KeyDownEvent, window, cx| {
                    let active_entry =
                        normalized_picker_entry(&keyboard_entries, state.read(cx).active_entry);
                    let set_active = |next_entry: Option<PickerPanelEntry>, cx: &mut App| {
                        state.update(cx, |state, cx| {
                            if state.active_entry != next_entry {
                                state.active_entry = next_entry;
                                cx.notify();
                            }
                        });
                    };
                    match event.keystroke.key.as_str() {
                        "down" => {
                            set_active(
                                moved_picker_entry(
                                    &keyboard_entries,
                                    active_entry,
                                    PickerMove::Next,
                                ),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "up" => {
                            set_active(
                                moved_picker_entry(
                                    &keyboard_entries,
                                    active_entry,
                                    PickerMove::Previous,
                                ),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "home" => {
                            set_active(
                                moved_picker_entry(
                                    &keyboard_entries,
                                    active_entry,
                                    PickerMove::First,
                                ),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "end" => {
                            set_active(
                                moved_picker_entry(
                                    &keyboard_entries,
                                    active_entry,
                                    PickerMove::Last,
                                ),
                                cx,
                            );
                            cx.stop_propagation();
                        }
                        "enter" | " " => {
                            if let Some(active_entry) = active_entry {
                                match active_entry {
                                    PickerPanelEntry::Item(index) => {
                                        if let Some(key) = item_keys.get(index).cloned() {
                                            if let Some(selection) = &selection {
                                                selection.select(cx, key.clone());
                                            }
                                            if let Some(handler) = &select_handler {
                                                handler(key, window, cx);
                                            }
                                            cx.stop_propagation();
                                        }
                                    }
                                    PickerPanelEntry::Action(index) => {
                                        if let (Some(key), Some(handler)) =
                                            (action_keys.get(index), action_handler.as_ref())
                                        {
                                            handler(key.clone(), window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .child(
                div()
                    .h(px(26.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_muted)
                            .child("SWITCH BRANCH"),
                    )
                    .child(
                        Icon::new(IconName::Folder)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    ),
            );

        for (index, item) in items.into_iter().enumerate() {
            let selected = selected_key
                .as_ref()
                .is_some_and(|selected_key| item.key == *selected_key);
            let key = item.key.clone();
            let handler = select_handler.clone();
            let selection = selection.clone();
            let state_for_hover = state.clone();
            let state_for_click = state.clone();
            let row_fg = if selected {
                theme.text
            } else {
                theme.text_secondary
            };
            let active = active_entry == Some(PickerPanelEntry::Item(index));

            panel = panel.child(
                div()
                    .id(("branch-option", index))
                    .min_h(px(34.0))
                    .px_2()
                    .py_1()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(radius::MD))
                    .role(Role::ListItem)
                    .text_color(row_fg)
                    .when(item_clickable, |this| this.cursor_pointer())
                    .when(selected, |this| this.bg(theme.selection))
                    .when(!selected && active, |this| this.bg(theme.hover))
                    .when(!selected && item_clickable, |this| {
                        this.hover(move |style| style.bg(theme.hover))
                    })
                    .on_hover(move |hovered, _window, cx| {
                        if item_clickable && *hovered {
                            state_for_hover.update(cx, |state, cx| {
                                if state.active_entry != Some(PickerPanelEntry::Item(index)) {
                                    state.active_entry = Some(PickerPanelEntry::Item(index));
                                    cx.notify();
                                }
                            });
                        }
                    })
                    .child(
                        div()
                            .w(px(16.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if selected {
                                Icon::new(IconName::Check)
                                    .size(IconSize::Small)
                                    .color(theme.accent)
                                    .into_any_element()
                            } else {
                                Icon::new(IconName::Folder)
                                    .size(IconSize::Small)
                                    .color(theme.text_muted)
                                    .into_any_element()
                            }),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(BORDER_WIDTH))
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(item.label),
                            )
                            .when_some(item.detail, |this, detail| {
                                this.child(
                                    div()
                                        .truncate()
                                        .text_size(px(11.0))
                                        .text_color(theme.text_muted)
                                        .child(detail),
                                )
                            }),
                    )
                    .when(item_clickable, |this| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
                            state_for_click.update(cx, |state, cx| {
                                if state.active_entry != Some(PickerPanelEntry::Item(index)) {
                                    state.active_entry = Some(PickerPanelEntry::Item(index));
                                    cx.notify();
                                }
                            });
                            if let Some(selection) = &selection {
                                selection.select(cx, key.clone());
                            }
                            if let Some(handler) = &handler {
                                handler(key.clone(), window, cx);
                            }
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        if !actions.is_empty() {
            panel = panel.child(
                div()
                    .my(px(space::XS))
                    .h(px(BORDER_WIDTH))
                    .w_full()
                    .bg(theme.border),
            );
        }

        for (index, action) in actions.into_iter().enumerate() {
            let handler = action_handler.clone();
            let key = action.key.clone();
            let state_for_hover = state.clone();
            let state_for_click = state.clone();
            let active = active_entry == Some(PickerPanelEntry::Action(index));
            panel = panel.child(
                div()
                    .id(("branch-picker-action", index))
                    .h(px(30.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(radius::MD))
                    .text_sm()
                    .role(Role::Button)
                    .text_color(if active {
                        theme.text
                    } else {
                        theme.text_secondary
                    })
                    .when(action_clickable, |this| this.cursor_pointer())
                    .when(active, |this| this.bg(theme.hover))
                    .when(action_clickable, |this| {
                        this.hover(move |style| style.bg(theme.hover).text_color(theme.text))
                    })
                    .on_hover(move |hovered, _window, cx| {
                        if action_clickable && *hovered {
                            state_for_hover.update(cx, |state, cx| {
                                if state.active_entry != Some(PickerPanelEntry::Action(index)) {
                                    state.active_entry = Some(PickerPanelEntry::Action(index));
                                    cx.notify();
                                }
                            });
                        }
                    })
                    .child(
                        Icon::new(action.icon)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(action.label))
                    .when_some(handler, |this, handler| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
                            state_for_click.update(cx, |state, cx| {
                                if state.active_entry != Some(PickerPanelEntry::Action(index)) {
                                    state.active_entry = Some(PickerPanelEntry::Action(index));
                                    cx.notify();
                                }
                            });
                            handler(key.clone(), window, cx);
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        panel.motion_slide_in(MotionDirection::FromTop, true)
    }
}

pub(super) fn default_picker_actions() -> Vec<PickerAction> {
    vec![
        PickerAction::new("branch:create", "Create branch", IconName::Plus),
        PickerAction::new("worktree:create", "New worktree", IconName::FolderPlus),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_picker_entry_prefers_selected_item() {
        let entries = vec![
            PickerKeyboardEntry {
                target: PickerPanelEntry::Item(0),
                selected: false,
            },
            PickerKeyboardEntry {
                target: PickerPanelEntry::Item(1),
                selected: true,
            },
            PickerKeyboardEntry {
                target: PickerPanelEntry::Action(0),
                selected: false,
            },
        ];

        assert_eq!(
            preferred_picker_entry(&entries),
            Some(PickerPanelEntry::Item(1))
        );
    }

    #[test]
    fn moved_picker_entry_wraps_across_items_and_actions() {
        let entries = vec![
            PickerKeyboardEntry {
                target: PickerPanelEntry::Item(0),
                selected: true,
            },
            PickerKeyboardEntry {
                target: PickerPanelEntry::Item(1),
                selected: false,
            },
            PickerKeyboardEntry {
                target: PickerPanelEntry::Action(0),
                selected: false,
            },
        ];

        assert_eq!(
            moved_picker_entry(&entries, None, PickerMove::First),
            Some(PickerPanelEntry::Item(0))
        );
        assert_eq!(
            moved_picker_entry(&entries, Some(PickerPanelEntry::Item(1)), PickerMove::Next),
            Some(PickerPanelEntry::Action(0))
        );
        assert_eq!(
            moved_picker_entry(
                &entries,
                Some(PickerPanelEntry::Action(0)),
                PickerMove::Next
            ),
            Some(PickerPanelEntry::Item(0))
        );
        assert_eq!(
            moved_picker_entry(
                &entries,
                Some(PickerPanelEntry::Item(0)),
                PickerMove::Previous
            ),
            Some(PickerPanelEntry::Action(0))
        );
    }
}
