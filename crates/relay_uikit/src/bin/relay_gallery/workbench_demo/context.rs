use gpui::{App, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder, px};
use relay_uikit::patterns::{
    Pane, PaneSurface, PaneWidth,
    navigation::{Tab, Tabs},
};
use relay_uikit::workbench::{DiffView, FileKind, FileView, TerminalSessionRow};
use relay_uikit::{
    Badge, IconButton, IconName, PanelHeader, StatusDot, TextInput, Theme, Tone,
    TreeRow, radius,
};

use super::{
    WorkbenchState,
    data::{DEMO_FILES, active_session},
};

pub(super) fn right_context(
    state: &WorkbenchState,
    window: &Window,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let tab = state.context_tab.get(cx);
    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .child(
            div().px_2().pt_2().child(
                Tabs::bound(
                    "ctx-tabs",
                    vec![
                        Tab::new("files", "Files").icon(IconName::FileText),
                        Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                        Tab::new("review", "Review")
                            .icon(IconName::MessageSquareText)
                            .count(3),
                    ],
                    state.context_tab.clone(),
                ),
            ),
        )
        .when(tab == "files", |this| {
            this.child(files_tab(state, window, cx))
        })
        .when(tab == "diff", |this| this.child(diff_tab()))
        .when(tab == "review", |this| {
            this.child(review_tab(state, theme, cx))
        });

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Chrome)
        .header(
            PanelHeader::new("Context")
                .icon(IconName::FileText)
                .trailing(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            IconButton::new("ctx-refresh", IconName::RefreshCw).on_click({
                                let filter = state.filter.clone();
                                move |_event, _window, cx| {
                                    filter.update(cx, |state| {
                                        state.clear();
                                        true
                                    });
                                }
                            }),
                        )
                        .child(IconButton::new("ctx-more", IconName::Ellipsis).on_click({
                            let context_tab = state.context_tab.clone();
                            move |_event, _window, cx| {
                                context_tab.set(cx, "review");
                            }
                        })),
                ),
        )
}

fn files_tab(
    state: &WorkbenchState,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let filter_focused = state.filter_focus.is_focused(window);
    let filter_text = state.filter.get(cx).value().to_lowercase();
    let files = DEMO_FILES
        .iter()
        .filter(|file| filter_text.is_empty() || file.name.to_lowercase().contains(&filter_text))
        .enumerate()
        .map(|(index, file)| {
            let mut row = TreeRow::new(("file", index), file.icon, file.name).depth(file.depth);
            if file.expandable {
                row = row.expandable(true);
            }
            if file.name == "workbench_demo.rs" {
                row = row.selected(true);
            }
            row.into_any_element()
        });

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(
            div().px_2().py_2().child(
                TextInput::bound("file-filter", state.filter_focus.clone(), state.filter.clone())
                    .placeholder("Filter files")
                    .leading_icon(IconName::Funnel)
                    .focused(filter_focused),
            ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px_2()
                .pb_2()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .children(files),
        )
}

fn diff_tab() -> impl IntoElement {
    div().flex_1().min_h_0().p_2().child(
        FileView::new(
            "crates/patterns/src/layout/shell/split_pane/mod.rs",
            FileKind::Diff,
            DiffView::from_text_diff(DIFF_OLD, DIFF_NEW),
        )
        .detail("+4 -1"),
    )
}

fn review_tab(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let session = active_session(state.active_session.get(cx));
    let route = state.route.clone();
    let context_tab = state.context_tab.clone();

    div()
        .flex_1()
        .min_h_0()
        .p_2()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            TerminalSessionRow::new("review-session", session.label, session.subtitle)
                .status(session.tone)
                .active(true)
                .on_click({
                    let route = route.clone();
                    let context_tab = context_tab.clone();
                    move |_event, _window, cx| {
                        route.set(cx, "terminal");
                        context_tab.set(cx, "files");
                    }
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(StatusDot::new(Tone::Warning))
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(theme.text)
                        .child("3 comments pending delivery"),
                )
                .child(Badge::new("DRAFT").tone(Tone::Warning).soft()),
        )
        .child(
            div()
                .p_2()
                .rounded(px(radius::MD))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .text_xs()
                .text_color(theme.text_secondary)
                .child("workbench_demo.rs: terminal launcher state should stay host-owned."),
        )
}

const DIFF_OLD: &str = r#"let center_and_context = SplitPane::new("center-context-split", center, right)
    .first_size(720.0)
    .min_sizes(560.0, 320.0);
"#;

const DIFF_NEW: &str = r#"let center_and_context = SplitPane::new("center-context-split", center, right)
    .first_size(state.terminal_split.first_size())
    .min_sizes(560.0, 320.0)
    .on_resize({
        let host = host.clone();
        move |next, _window, cx| {
            host.update(cx, |this, cx| {
                if this.state.terminal_split.resize_to(next) {
                    cx.notify();
                }
            });
        }
    });
"#;
