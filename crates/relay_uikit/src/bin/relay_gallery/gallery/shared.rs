use gpui::{AnyElement, App, Context, Entity, FocusHandle, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay::{Binding, ReactiveAppExt, Signal};
use relay_uikit::workbench::{BranchOption, BranchSelector};
use relay_uikit::{
    ActiveTheme, Icon, IconButton, IconName, IconSize, StatusDot, TextInput, TextInputState, Theme,
    Tone, radius, space,
};

use super::GalleryScenesApp;

pub(super) fn section<T: IntoElement>(
    cx: &mut Context<GalleryScenesApp>,
    title: &str,
    body: T,
) -> impl IntoElement + use<T> {
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
    _host: &Entity<GalleryScenesApp>,
    cx: &mut App,
    branch_choice: &Binding<&'static str>,
    picker_open: &Binding<bool>,
    actions_open: &Binding<bool>,
    branch_event: &Signal<String>,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(branch_selector(cx, branch_choice, picker_open, branch_event))
        .child(branch_actions_button(cx, actions_open, branch_event))
        .into_any_element()
}

fn branch_selector(
    cx: &mut App,
    branch_choice: &Binding<&'static str>,
    open: &Binding<bool>,
    branch_event: &Signal<String>,
) -> AnyElement {
    // Snapshot reads are untracked: BranchSelector::new takes a value, not a
    // binding, so subscribing here would only cause redundant re-renders. The
    // component refreshes via the surrounding tracked render instead.
    let (selected_key, open_val) = cx.untrack(|cx| (branch_choice.get(cx), open.get(cx)));
    BranchSelector::new(
        "gallery-branch-selector",
        selected_key,
        vec![
            BranchOption::new("main", "main").detail("default branch"),
            BranchOption::new("ui-kit-branch-controls", "ui-kit/branch-controls")
                .detail("current worktree"),
            BranchOption::new("terminal-rio-adapter", "terminal/rio-adapter")
                .detail("terminal renderer spike"),
            BranchOption::new("review-viewers", "review/viewers").detail("diff and file views"),
        ],
    )
    .open(open_val)
    .on_toggle({
        let open = open.clone();
        move |_event, _window, cx| {
            open.update(cx, |v| {
                *v = !*v;
                true
            });
        }
    })
    .on_select({
        let choice = branch_choice.clone();
        let open = open.clone();
        let event = branch_event.clone();
        move |key, _window, cx| {
            choice.set(cx, key);
            open.set(cx, false);
            event.set(cx, format!("Switched to {key}"));
        }
    })
    .on_action({
        let open = open.clone();
        let event = branch_event.clone();
        move |key, _window, cx| {
            open.set(cx, false);
            event.set(cx, match key {
                "branch:create" => "Create branch requested".into(),
                "worktree:create" => "New worktree requested".into(),
                _ => format!("Branch picker action: {key}"),
            });
        }
    })
    .on_dismiss({
        let open = open.clone();
        move |_window, cx| {
            open.set(cx, false);
        }
    })
    .into_any_element()
}

fn branch_actions_button(
    cx: &mut App,
    open: &Binding<bool>,
    _branch_event: &Signal<String>,
) -> AnyElement {
    let open_val = cx.untrack(|cx| open.get(cx));
    IconButton::new("gallery-branch-actions", IconName::Ellipsis)
        .active(open_val)
        .on_click({
            let open = open.clone();
            move |_event, _window, cx| {
                open.update(cx, |v| {
                    *v = !*v;
                    true
                });
            }
        })
        .into_any_element()
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
