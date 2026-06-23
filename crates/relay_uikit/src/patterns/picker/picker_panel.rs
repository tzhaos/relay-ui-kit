use std::hash::Hash;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{SelectionSource, SharedActionHandler},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, radius, space},
};
use gpui::{
    App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use super::picker_types::{PickerAction, PickerOption};

pub(super) fn branch_picker_panel<K>(
    selected_key: Option<K>,
    items: Vec<PickerOption<K>>,
    actions: Vec<PickerAction>,
    selection: Option<SelectionSource<K>>,
    select_handler: Option<SharedActionHandler<K>>,
    action_handler: Option<SharedActionHandler<String>>,
) -> impl IntoElement
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    PickerPanel {
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
    selected_key: Option<K>,
    items: Vec<PickerOption<K>>,
    actions: Vec<PickerAction>,
    selection: Option<SelectionSource<K>>,
    select_handler: Option<SharedActionHandler<K>>,
    action_handler: Option<SharedActionHandler<String>>,
}

impl<K> RenderOnce for PickerPanel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selection = self.selection;
        let selected_key = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(self.selected_key);
        let select_handler = self.select_handler;
        let mut panel = div()
            .id("branch-picker-panel")
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

        for (index, item) in self.items.into_iter().enumerate() {
            let selected = selected_key
                .as_ref()
                .is_some_and(|selected_key| item.key == *selected_key);
            let key = item.key.clone();
            let handler = select_handler.clone();
            let selection = selection.clone();
            let row_fg = if selected {
                theme.text
            } else {
                theme.text_secondary
            };
            let clickable = selection.is_some() || handler.is_some();

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
                    .text_color(row_fg)
                    .when(clickable, |this| this.cursor_pointer())
                    .when(selected, |this| this.bg(theme.selection))
                    .when(!selected, |this| {
                        this.hover(move |style| style.bg(theme.hover))
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
                    .when(clickable, |this| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
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

        if !self.actions.is_empty() {
            panel = panel.child(
                div()
                    .my(px(space::XS))
                    .h(px(BORDER_WIDTH))
                    .w_full()
                    .bg(theme.border),
            );
        }

        for (index, action) in self.actions.into_iter().enumerate() {
            let handler = self.action_handler.clone();
            let key = action.key.clone();
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
                    .text_color(theme.text_secondary)
                    .cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).text_color(theme.text))
                    .child(
                        Icon::new(action.icon)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(action.label))
                    .when_some(handler, |this, handler| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
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
