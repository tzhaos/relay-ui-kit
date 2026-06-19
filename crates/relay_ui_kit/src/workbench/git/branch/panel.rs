use gpui::{
    App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    components::overlay::overlay,
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, radius, space},
};

use super::{
    selector::SelectHandler,
    types::{BranchOption, BranchPickerAction},
};

pub(super) fn branch_picker_panel(
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    select_handler: Option<SelectHandler>,
    action_handler: Option<SelectHandler>,
) -> impl IntoElement {
    overlay(
        BranchPickerPanel {
            selected_key,
            branches,
            actions,
            select_handler,
            action_handler,
        }
        .into_any_element(),
    )
    .offset(0.0, 34.0)
}

#[derive(IntoElement)]
struct BranchPickerPanel {
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    select_handler: Option<SelectHandler>,
    action_handler: Option<SelectHandler>,
}

impl RenderOnce for BranchPickerPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut panel = div()
            .id("branch-picker-panel")
            .w(px(320.0))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(2.0))
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
                        Icon::new(IconName::GitBranch)
                            .size(IconSize::Small)
                            .color(theme.text_muted),
                    ),
            );

        for (index, branch) in self.branches.into_iter().enumerate() {
            let handler = self.select_handler.clone();
            let selected = branch.key == self.selected_key;
            let key = branch.key;
            let row_fg = if selected {
                theme.text
            } else {
                theme.text_secondary
            };

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
                    .cursor_pointer()
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
                                Icon::new(IconName::GitBranch)
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
                            .gap(px(1.0))
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(branch.label),
                            )
                            .when_some(branch.detail, |this, detail| {
                                this.child(
                                    div()
                                        .truncate()
                                        .text_size(px(11.0))
                                        .text_color(theme.text_muted)
                                        .child(detail),
                                )
                            }),
                    )
                    .when_some(handler, |this, handler| {
                        this.on_click(move |_event: &ClickEvent, window, cx| {
                            handler(key, window, cx);
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        if !self.actions.is_empty() {
            panel = panel.child(div().my(px(space::XS)).h(px(1.0)).w_full().bg(theme.border));
        }

        for (index, action) in self.actions.into_iter().enumerate() {
            let handler = self.action_handler.clone();
            let key = action.key;
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
                            handler(key, window, cx);
                            cx.stop_propagation();
                        })
                    }),
            );
        }

        panel.motion_slide_in(MotionDirection::FromTop, true)
    }
}

pub(super) fn default_picker_actions() -> Vec<BranchPickerAction> {
    vec![
        BranchPickerAction::new("branch:create", "Create branch", IconName::Plus),
        BranchPickerAction::new("worktree:create", "New worktree", IconName::FolderPlus),
    ]
}
