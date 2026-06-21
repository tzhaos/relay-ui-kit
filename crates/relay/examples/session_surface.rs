//! SessionSurface - a workbench-like GPUI surface built with Relay primitives.
//!
//! The host owns a dynamic session list, retains one GPUI entity per row with
//! `KeyedSubViews`, and drives selection with host-level keyboard handling.
//!
//! Run with `cargo run -p relay --example session_surface`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    KeyedSubViews, ReactiveAppExt, ReactiveView, Selector, Signal, SignalVecExt, init,
    view::reactive_render,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionStatus {
    Waiting,
    Running,
    Done,
}

impl SessionStatus {
    fn label(self) -> &'static str {
        match self {
            SessionStatus::Waiting => "waiting",
            SessionStatus::Running => "running",
            SessionStatus::Done => "done",
        }
    }

    fn color(self) -> u32 {
        match self {
            SessionStatus::Waiting => 0xf59e0b,
            SessionStatus::Running => 0x22c55e,
            SessionStatus::Done => 0x94a3b8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Session {
    id: u64,
    title: String,
    agent: &'static str,
    branch: &'static str,
    status: SessionStatus,
    commands: u32,
}

impl Session {
    fn new(id: u64, title: impl Into<String>, agent: &'static str, branch: &'static str) -> Self {
        Self {
            id,
            title: title.into(),
            agent,
            branch,
            status: SessionStatus::Waiting,
            commands: 0,
        }
    }
}

struct SessionSurface {
    sessions: Signal<Vec<Session>>,
    selection: Selector<u64>,
    rows: KeyedSubViews<u64, SessionRow>,
    last_action: Signal<String>,
    next_id: u64,
}

impl SessionSurface {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            sessions: cx.signal(vec![
                Session::new(1, "Implement relay core", "codex", "relay/core"),
                Session::new(2, "Review GPUI cache boundary", "codex", "gpui/cache"),
                {
                    let mut session = Session::new(3, "Ship gallery migration", "cargo", "uikit");
                    session.status = SessionStatus::Done;
                    session
                },
            ]),
            selection: cx.selector(Some(1)),
            rows: KeyedSubViews::new(),
            last_action: cx.signal("ready".to_string()),
            next_id: 4,
        }
    }

    fn add_session(&mut self, cx: &mut App) {
        let id = self.next_id;
        self.next_id += 1;
        let session = Session::new(id, format!("Session {id}"), "codex", "scratch");

        cx.batch(|cx| {
            self.sessions.push(cx, session);
            self.selection.select(cx, id);
            self.last_action.set(cx, format!("created session {id}"));
        });
    }

    fn activate_selected(&self, cx: &mut App) -> bool {
        let Some(selected) = self.selection.get_untracked() else {
            return false;
        };
        let mut activated = None;

        cx.batch(|cx| {
            self.sessions.update(cx, |sessions| {
                let Some(session) = sessions.iter_mut().find(|session| session.id == selected)
                else {
                    return false;
                };
                session.status = SessionStatus::Running;
                session.commands += 1;
                activated = Some(session.title.clone());
                true
            });

            if let Some(title) = &activated {
                self.last_action.set(cx, format!("ran {title}"));
            }
        });

        activated.is_some()
    }

    fn close_selected(&self, cx: &mut App) -> bool {
        let Some(selected) = self.selection.get_untracked() else {
            return false;
        };
        let (found, remaining) = self.sessions.peek(|sessions| {
            (
                sessions.iter().any(|session| session.id == selected),
                sessions
                    .iter()
                    .filter(|session| session.id != selected)
                    .map(|session| session.id)
                    .collect::<Vec<_>>(),
            )
        });

        if !found {
            self.selection.clear(cx);
            return false;
        }

        let next_selection = remaining.first().copied();
        cx.batch(|cx| {
            self.sessions.retain(cx, |session| session.id != selected);
            self.selection.set(cx, next_selection);
            self.last_action
                .set(cx, format!("closed session {selected}"));
        });
        true
    }

    fn reverse_sessions(&self, cx: &mut App) {
        self.sessions.update(cx, |sessions| {
            if sessions.len() <= 1 {
                false
            } else {
                sessions.reverse();
                true
            }
        });
    }

    fn select_next_session(&self, cx: &mut App) {
        self.sessions.peek(|sessions| {
            self.selection
                .select_next_by(cx, sessions, |session| session.id);
        });
    }

    fn select_previous_session(&self, cx: &mut App) {
        self.sessions.peek(|sessions| {
            self.selection
                .select_previous_by(cx, sessions, |session| session.id);
        });
    }

    fn select_first_session(&self, cx: &mut App) {
        self.sessions.peek(|sessions| {
            self.selection
                .select_first_by(cx, sessions, |session| session.id);
        });
    }

    fn select_last_session(&self, cx: &mut App) {
        self.sessions.peek(|sessions| {
            self.selection
                .select_last_by(cx, sessions, |session| session.id);
        });
    }

    fn handle_key_down(&self, event: &KeyDownEvent, cx: &mut App) -> bool {
        match event.keystroke.key.as_str() {
            "arrow-down" => {
                self.select_next_session(cx);
                true
            }
            "arrow-up" => {
                self.select_previous_session(cx);
                true
            }
            "home" => {
                self.select_first_session(cx);
                true
            }
            "end" => {
                self.select_last_session(cx);
                true
            }
            "enter" => self.activate_selected(cx),
            "backspace" | "delete" => self.close_selected(cx),
            _ => false,
        }
    }

    fn selected_label(&self, cx: &App) -> String {
        let selected = self.selection.get(cx);
        self.sessions.read(cx, |sessions| {
            selected
                .and_then(|selected| sessions.iter().find(|session| session.id == selected))
                .map_or_else(|| "No session".to_string(), |session| session.title.clone())
        })
    }

    #[cfg(test)]
    fn session_ids(&self) -> Vec<u64> {
        self.sessions
            .peek(|sessions| sessions.iter().map(|session| session.id).collect())
    }
}

impl ReactiveView for SessionSurface {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sessions = self.sessions.get(cx);
        let running = sessions
            .iter()
            .filter(|session| session.status == SessionStatus::Running)
            .count();
        self.selection
            .reconcile_keys_by(cx, &sessions, |session| session.id);

        self.rows.sync(
            cx,
            sessions,
            |session| session.id,
            |session, cx| SessionRow::new(session, self.selection.clone(), cx),
            |session, row, _cx| row.update_session(session),
        );

        div()
            .id("session-surface")
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .tab_index(0)
            .bg(rgb(0x18181b))
            .text_color(rgb(0xf8fafc))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if this.handle_key_down(event, cx) {
                    cx.stop_propagation();
                }
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().text_lg().child("Session surface"))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Selected: {}", self.selected_label(cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(action_button("first", "First").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_first_session(cx);
                        },
                    )))
                    .child(action_button("previous", "Previous").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_previous_session(cx);
                        },
                    )))
                    .child(
                        action_button("next", "Next").on_click(cx.listener(|this, _, _, cx| {
                            this.select_next_session(cx);
                        })),
                    )
                    .child(
                        action_button("last", "Last").on_click(cx.listener(|this, _, _, cx| {
                            this.select_last_session(cx);
                        })),
                    )
                    .child(
                        action_button("run", "Run").on_click(cx.listener(|this, _, _, cx| {
                            this.activate_selected(cx);
                        })),
                    )
                    .child(action_button("close", "Close").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.close_selected(cx);
                        },
                    )))
                    .child(action_button("reverse", "Reverse").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.reverse_sessions(cx);
                        },
                    )))
                    .child(
                        action_button("add", "Add").on_click(cx.listener(|this, _, _, cx| {
                            this.add_session(cx);
                        })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(info_pill("Rows", self.rows.len().to_string()))
                    .child(info_pill("Running", running.to_string()))
                    .child(info_pill("Last action", self.last_action.get(cx))),
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

impl Render for SessionSurface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct SessionRow {
    session: Session,
    selection: Selector<u64>,
    expanded: Signal<bool>,
}

impl SessionRow {
    fn new(session: &Session, selection: Selector<u64>, cx: &mut Context<Self>) -> Self {
        Self {
            session: session.clone(),
            selection,
            expanded: cx.signal(false),
        }
    }

    fn update_session(&mut self, session: &Session) -> bool {
        if self.session == *session {
            false
        } else {
            self.session = session.clone();
            true
        }
    }

    #[cfg(test)]
    fn toggle_expanded(&self, cx: &mut App) {
        self.expanded.update(cx, |expanded| {
            *expanded = !*expanded;
            true
        });
    }
}

impl ReactiveView for SessionRow {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selection.is_selected(cx, self.session.id);
        let expanded = self.expanded.get(cx);
        let selection = self.selection.clone();
        let expanded_signal = self.expanded.clone();
        let id = self.session.id;
        let bg = if selected { 0x1e3a8a } else { 0x27272a };

        div()
            .id(format!("session-row-{id}"))
            .min_h(px(58.0))
            .px_3()
            .py_2()
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(6.0))
            .border_1()
            .border_color(if selected {
                rgb(0x60a5fa)
            } else {
                rgb(0x3f3f46)
            })
            .bg(rgb(bg))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x334155)))
            .on_click(move |_, _window, cx| {
                selection.select(cx, id);
                cx.stop_propagation();
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(self.session.title.clone()),
                            )
                            .child(div().truncate().text_xs().text_color(rgb(0xa1a1aa)).child(
                                format!("{} on {}", self.session.agent, self.session.branch),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(status_chip(self.session.status))
                            .child(
                                action_button(
                                    "row-details",
                                    if expanded { "Hide" } else { "Details" },
                                )
                                .on_click(move |_, _window, cx| {
                                    expanded_signal.update(cx, |expanded| {
                                        *expanded = !*expanded;
                                        true
                                    });
                                    cx.stop_propagation();
                                }),
                            ),
                    ),
            )
            .when(expanded, |this| {
                this.child(
                    div()
                        .pl_1()
                        .text_xs()
                        .text_color(rgb(0xcbd5e1))
                        .child(format!("Commands run: {}", self.session.commands)),
                )
            })
            .into_any_element()
    }
}

impl Render for SessionRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn action_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded(px(4.0))
        .bg(rgb(0x3b82f6))
        .hover(|style| style.bg(rgb(0x2563eb)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn info_pill(label: &'static str, value: String) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded(px(999.0))
        .bg(rgb(0x27272a))
        .text_xs()
        .text_color(rgb(0xd4d4d8))
        .child(format!("{label}: {value}"))
}

fn status_chip(status: SessionStatus) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded(px(999.0))
        .bg(rgb(0x18181b))
        .text_xs()
        .text_color(rgb(status.color()))
        .child(status.label())
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(760.0), px(420.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(SessionSurface::new),
        );
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}

#[cfg(test)]
mod tests {
    use gpui::{Entity, EntityId, Keystroke, TestApp};

    use super::*;

    fn key(name: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                key: name.to_string(),
                ..Default::default()
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    fn row_ids(rows: &KeyedSubViews<u64, SessionRow>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    fn row_entity(surface: &SessionSurface, key: u64) -> Entity<SessionRow> {
        match surface.rows.get(&key) {
            Some(row) => row.clone_entity(),
            None => panic!("missing session row {key}"),
        }
    }

    #[test]
    fn session_surface_reuses_rows_when_sessions_reorder() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SessionSurface::new(cx));
        let root = window.root();

        window.draw();
        let initial_rows = app.update_entity(&root, |surface, _cx| row_ids(&surface.rows));

        app.update_entity(&root, |surface, cx| {
            surface.reverse_sessions(cx);
        });
        window.draw();

        let updated_rows = app.update_entity(&root, |surface, _cx| row_ids(&surface.rows));
        assert_eq!(
            updated_rows,
            vec![
                (3, initial_rows[2].1),
                (2, initial_rows[1].1),
                (1, initial_rows[0].1),
            ]
        );
    }

    #[test]
    fn session_surface_row_state_survives_reorder() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SessionSurface::new(cx));
        let root = window.root();

        window.draw();
        let row = app.update_entity(&root, |surface, _cx| row_entity(surface, 2));
        app.update_entity(&row, |row, cx| {
            row.toggle_expanded(cx);
        });

        app.update_entity(&root, |surface, cx| {
            surface.reverse_sessions(cx);
        });
        window.draw();

        let row_after = app.update_entity(&root, |surface, _cx| row_entity(surface, 2));
        let expanded = app.update_entity(&row_after, |row, _cx| row.expanded.get_untracked());
        assert_eq!(row.entity_id(), row_after.entity_id());
        assert!(expanded);
    }

    #[test]
    fn session_surface_keyboard_navigation_runs_selected_session() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| SessionSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            assert!(surface.handle_key_down(&key("end"), cx));
            assert!(surface.handle_key_down(&key("enter"), cx));
            assert!(surface.handle_key_down(&key("home"), cx));
        });

        let (selection, sessions, last_action) = app.update_entity(&root, |surface, _cx| {
            (
                surface.selection.get_untracked(),
                surface.sessions.get_untracked(),
                surface.last_action.get_untracked(),
            )
        });

        assert_eq!(selection, Some(1));
        assert_eq!(sessions[2].status, SessionStatus::Running);
        assert_eq!(sessions[2].commands, 1);
        assert_eq!(last_action, "ran Ship gallery migration");
    }

    #[test]
    fn session_surface_keyboard_delete_removes_selected_session() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| SessionSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            assert!(surface.handle_key_down(&key("end"), cx));
            assert!(surface.handle_key_down(&key("delete"), cx));
        });

        let (selection, ids) = app.update_entity(&root, |surface, _cx| {
            (surface.selection.get_untracked(), surface.session_ids())
        });

        assert_eq!(selection, Some(1));
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn session_surface_keyboard_ignores_unhandled_keys() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| SessionSurface::new(cx));
        let root = window.root();

        let handled = app.update_entity(&root, |surface, cx| {
            surface.handle_key_down(&key("tab"), cx)
        });

        assert!(!handled);
    }
}
