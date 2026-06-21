use gpui::{
    AnyElement, AnyView, App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, px,
};
use relay::{KeyedSubViews, ReactiveAppExt, ReactiveView, Signal, view::reactive_render};
use relay_uikit::patterns::ScrollSurface;
use relay_uikit::{
    ActiveTheme, Button, ButtonVariant, IconButton, IconName, IconSize, Label, LabelSize, ListItem,
    ListItemSpacing, StatusDot, Theme, Tone, TreeRow, radius,
};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{scene_stack, section, strip},
};

pub(super) fn render(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(
            cx,
            "Long text",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(long_labels())
                .child(long_file_tree()),
        ))
        .child(section(
            cx,
            "Disabled and quiet states",
            strip()
                .child(
                    Button::new("stress-disabled-primary", "Primary Action")
                        .primary()
                        .icon(IconName::Play)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-secondary", "Archive")
                        .variant(ButtonVariant::Secondary)
                        .icon(IconName::Archive)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-ghost", "Refresh")
                        .ghost()
                        .icon(IconName::RefreshCw)
                        .disabled(true),
                ),
        ))
        .child(section(
            cx,
            "Disabled icon buttons",
            strip()
                .child(
                    IconButton::new("stress-ib-disabled", IconName::Plus)
                        .size(IconSize::Small)
                        .disabled(true),
                )
                .child(
                    IconButton::new("stress-ib-active-disabled", IconName::PanelLeft)
                        .active(true)
                        .size(IconSize::Small)
                        .disabled(true),
                )
                .child(
                    IconButton::new("stress-ib-active", IconName::Settings)
                        .active(true)
                        .size(IconSize::Small),
                ),
        ))
        .child(section(cx, "Dense rows", long_list(theme)))
        .child(section(
            cx,
            "Keyed session list",
            cached_session_list(state.stress_session_list.clone()),
        ))
        .child(section(cx, "Scroll surface", scroll_surface_sample(theme)))
}

fn long_labels() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(Label::new(
            "Repair terminal focus after switching between a Codex session and a plain shell in a nested worktree",
        ))
        .child(
            Label::new("Check long review note delivery state with an extremely verbose label that exceeds any reasonable container width constraint set by the layout system")
                .size(LabelSize::Small),
        )
        .child(
            Label::new("short but dense row")
                .size(LabelSize::XSmall),
        )
}

fn long_list(theme: Theme) -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            ListItem::new("stress-dense-1")
                .child(div().truncate().text_sm().text_color(theme.text).child(
                    "Repair terminal focus after switching between a Codex session and a plain shell",
                ))
                .end_slot(div().text_size(px(11.0)).text_color(Theme::light().danger).child("RUNNING")),
        )
        .child(
            ListItem::new("stress-dense-2")
                .child(div().truncate().text_sm().text_color(theme.text).child(
                    "Check long review note delivery state with verbose label text",
                ))
                .end_slot(div().text_size(px(11.0)).text_color(Theme::light().warning).child("WAITING")),
        )
}

fn cached_session_list(list: Entity<StressSessionList>) -> AnyElement {
    let view: AnyView = list.into();
    view.cached(gpui::StyleRefinement::default().w_full())
        .into_any_element()
}

#[derive(Clone, PartialEq, Eq)]
struct StressSession {
    id: u64,
    title: String,
    detail: String,
    tone: Tone,
    active: bool,
}

impl StressSession {
    fn new(id: u64, title: impl Into<String>, detail: impl Into<String>, tone: Tone) -> Self {
        Self {
            id,
            title: title.into(),
            detail: detail.into(),
            tone,
            active: false,
        }
    }

    fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

pub(super) struct StressSessionList {
    sessions: Signal<Vec<StressSession>>,
    rows: KeyedSubViews<u64, StressSessionRow>,
    next_id: u64,
}

impl StressSessionList {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            sessions: cx.signal(vec![
                StressSession::new(
                    1,
                    "Terminal focus repair",
                    "relay-ui-kit / session-a",
                    Tone::Accent,
                )
                .active(true),
                StressSession::new(
                    2,
                    "Review note sync",
                    "relay-ui-kit / session-b",
                    Tone::Info,
                ),
                StressSession::new(
                    3,
                    "Long artifact preview",
                    "relay-ui-kit / session-c",
                    Tone::Warning,
                ),
                StressSession::new(
                    4,
                    "Idle command runner",
                    "relay-ui-kit / session-d",
                    Tone::Muted,
                ),
            ]),
            rows: KeyedSubViews::new(),
            next_id: 5,
        }
    }

    fn add_session(&mut self, cx: &mut App) {
        let id = self.next_id;
        self.next_id += 1;
        self.sessions.update(cx, |sessions| {
            sessions.push(StressSession::new(
                id,
                format!("Generated session {id:02}"),
                "relay-ui-kit / generated",
                Tone::Secondary,
            ));
            true
        });
    }

    fn activate_next(&self, cx: &mut App) {
        self.sessions.update(cx, |sessions| {
            let Some(current) = sessions.iter().position(|session| session.active) else {
                if let Some(first) = sessions.first_mut() {
                    first.active = true;
                    return true;
                }
                return false;
            };
            let next = (current + 1) % sessions.len();
            sessions[current].active = false;
            sessions[next].active = true;
            true
        });
    }

    fn rotate(&self, cx: &mut App) {
        self.sessions.update(cx, |sessions| {
            if sessions.len() < 2 {
                return false;
            }
            sessions.rotate_left(1);
            true
        });
    }
}

impl ReactiveView for StressSessionList {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sessions = self.sessions.get(cx);

        self.rows.sync(
            cx,
            sessions,
            |session| session.id,
            |session, _cx| StressSessionRow::new(session),
            |session, row, _cx| row.update_session(session),
        );

        div()
            .w(px(520.0))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("stress-keyed-activate", "Activate Next")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.activate_next(cx);
                            })),
                    )
                    .child(
                        Button::new("stress-keyed-rotate", "Rotate")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.rotate(cx);
                            })),
                    )
                    .child(
                        Button::new("stress-keyed-add", "Add")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.add_session(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.rows.cached(gpui::StyleRefinement::default().w_full())),
            )
            .into_any_element()
    }
}

impl Render for StressSessionList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct StressSessionRow {
    session: StressSession,
}

impl StressSessionRow {
    fn new(session: &StressSession) -> Self {
        Self {
            session: session.clone(),
        }
    }

    fn update_session(&mut self, session: &StressSession) -> bool {
        if self.session == *session {
            false
        } else {
            self.session = session.clone();
            true
        }
    }
}

impl ReactiveView for StressSessionRow {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let session = &self.session;

        ListItem::new(format!("stress-keyed-session-{}", session.id))
            .spacing(ListItemSpacing::Dense)
            .selected(session.active)
            .start_slot(StatusDot::new(session.tone))
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .text_color(theme.text)
                            .child(session.title.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(session.detail.clone()),
                    ),
            )
            .end_slot(
                div()
                    .text_size(px(11.0))
                    .text_color(session.tone.fg(&theme))
                    .child(if session.active { "ACTIVE" } else { "READY" }),
            )
            .into_any_element()
    }
}

impl Render for StressSessionRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn long_file_tree() -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .child(
            TreeRow::new("stress-tree-root", IconName::Folder, "crates")
                .expandable(true)
                .depth(0),
        )
        .child(
            TreeRow::new(
                "stress-tree-deep",
                IconName::Folder,
                "relay_uikit/src/components/controls/segmented_control_very_long_name.rs",
            )
            .depth(1),
        )
        .child(
            TreeRow::new(
                "stress-tree-file",
                IconName::FileText,
                "terminal_session_history_projection_with_extremely_long_name.rs",
            )
            .depth(2)
            .selected(true),
        )
        .child(TreeRow::new(
            "stress-tree-diff",
            IconName::FileDiff,
            "components/list/tree_view_long_item_name.rs",
        ))
}

fn scroll_surface_sample(theme: Theme) -> impl IntoElement {
    div().h(px(180.0)).child(ScrollSurface::new(
        "stress-scroll-surface",
        div()
            .flex()
            .flex_col()
            .gap(px(1.0))
            .children((0..24).map(move |index| {
                div()
                    .h(px(28.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(px(radius::MD))
                    .bg(if index % 2 == 0 {
                        theme.panel
                    } else {
                        theme.panel_alt
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(format!("Session history row {index:02}")),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(if index % 3 == 0 { "active" } else { "idle" }),
                    )
            })),
    ))
}
