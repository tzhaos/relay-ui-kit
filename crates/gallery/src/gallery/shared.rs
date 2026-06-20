use gpui::{
    Context, Entity, FocusHandle, FontWeight, IntoElement, ParentElement, Styled, div,
    prelude::FluentBuilder, px,
};
use relay_foundation::{
    ActiveTheme, Icon, IconButton, IconName, IconSize, StatusDot, TextInput, TextInputState, Theme,
    Tone, overlay, radius, space,
};
use relay_workbench::{BranchActionKind, BranchActionsMenu, BranchOption, BranchSelector};

use super::GalleryScenesApp;

pub(super) fn section(
    cx: &mut Context<GalleryScenesApp>,
    title: &str,
    body: impl IntoElement,
) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(title.to_uppercase()),
        )
        .child(
            div()
                .p_3()
                .rounded(px(radius::LG))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .child(body),
        )
}

pub(super) fn scene_stack() -> gpui::Div {
    div().flex().flex_col().gap(px(space::XL))
}

pub(super) fn strip() -> gpui::Div {
    div().flex().items_center().gap_3().flex_wrap()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_input_field(
    host: &Entity<GalleryScenesApp>,
    id: &'static str,
    input: &TextInputState,
    focus: FocusHandle,
    focused: bool,
    icon: Option<IconName>,
    placeholder: &'static str,
) -> impl IntoElement {
    let host = host.clone();
    let is_name = id.contains("name");
    let mut field = TextInput::new(id, focus, input)
        .placeholder(placeholder)
        .focused(focused)
        .on_key(move |event, _window, cx| {
            host.update(cx, |this, cx| {
                let target = if is_name {
                    &mut this.state.name_input
                } else {
                    &mut this.state.search_input
                };
                let action = target.handle_key(event);
                if action.should_notify() {
                    cx.notify();
                }
            });
        });
    if let Some(icon) = icon {
        field = field.leading_icon(icon);
    }
    field
}

pub(super) fn branch_controls(
    host: &Entity<GalleryScenesApp>,
    selected: &'static str,
    picker_open: bool,
    actions_open: bool,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(branch_selector(host, selected, picker_open))
        .child(
            div()
                .relative()
                .child(branch_actions_button(host, actions_open))
                .when(actions_open, |this| this.child(branch_actions_menu(host))),
        )
}

fn branch_selector(
    host: &Entity<GalleryScenesApp>,
    selected: &'static str,
    open: bool,
) -> impl IntoElement {
    BranchSelector::new(
        "gallery-branch-selector",
        selected,
        vec![
            BranchOption::new("main", "main").detail("default branch"),
            BranchOption::new("ui-kit-branch-controls", "ui-kit/branch-controls")
                .detail("current worktree"),
            BranchOption::new("terminal-rio-adapter", "terminal/rio-adapter")
                .detail("terminal renderer spike"),
            BranchOption::new("review-viewers", "review/viewers").detail("diff and file views"),
        ],
    )
    .open(open)
    .on_toggle({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.branch_picker_open = !open;
                this.state.branch_actions_open = false;
                cx.notify();
            });
        }
    })
    .on_select({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.branch_choice = key;
                this.state.branch_picker_open = false;
                this.state.branch_event = format!("Switched to {key}");
                cx.notify();
            });
        }
    })
    .on_action({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.branch_picker_open = false;
                this.state.branch_event = match key {
                    "branch:create" => "Create branch requested".into(),
                    "worktree:create" => "New worktree requested".into(),
                    _ => format!("Branch picker action: {key}"),
                };
                cx.notify();
            });
        }
    })
    .on_dismiss({
        let host = host.clone();
        move |_window, cx| {
            host.update(cx, |this, cx| {
                this.state.branch_picker_open = false;
                cx.notify();
            });
        }
    })
}

fn branch_actions_button(host: &Entity<GalleryScenesApp>, open: bool) -> impl IntoElement {
    IconButton::new("gallery-branch-actions", IconName::Ellipsis)
        .active(open)
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.branch_actions_open = !open;
                    this.state.branch_picker_open = false;
                    cx.notify();
                });
            }
        })
}

fn branch_actions_menu(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    overlay(
        BranchActionsMenu::new("gallery-branch-actions-menu").on_select({
            let host = host.clone();
            move |action: BranchActionKind, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.branch_actions_open = false;
                    this.state.branch_event = action.label().to_string();
                    cx.notify();
                });
            }
        }),
    )
    .offset(0.0, 32.0)
    .on_dismiss({
        let host = host.clone();
        move |_window, cx| {
            host.update(cx, |this, cx| {
                this.state.branch_actions_open = false;
                cx.notify();
            });
        }
    })
}

pub(super) fn dot_label(theme: Theme, tone: Tone, label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(StatusDot::new(tone))
        .child(
            div()
                .text_sm()
                .text_color(theme.text_secondary)
                .child(label.to_string()),
        )
}

pub(super) fn icon_sample(theme: Theme, name: IconName) -> impl IntoElement {
    div()
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .bg(theme.panel_alt)
        .border_1()
        .border_color(theme.border)
        .child(
            Icon::new(name)
                .size(IconSize::Medium)
                .color(theme.text_secondary),
        )
}
