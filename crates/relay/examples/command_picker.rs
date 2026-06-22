//! CommandPicker - command/picker state built from Relay primitives.
//!
//! The host owns command data, query state, selected key order, and execution.
//! Relay provides the reactive pieces (`Binding`, `Memo`, `Selector`), while
//! concrete command/picker UI remains an app or component-layer decision.
//!
//! Run with `cargo run -p relay --example command_picker`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Render, Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{
    Binding, Memo, ReactiveAppExt, ReactiveView, SelectedItemExt, Selector, Signal, init,
    view::reactive_render,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommandItem {
    id: &'static str,
    title: &'static str,
    group: &'static str,
    description: &'static str,
}

impl CommandItem {
    const fn new(
        id: &'static str,
        title: &'static str,
        group: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            id,
            title,
            group,
            description,
        }
    }
}

struct CommandPickerSurface {
    commands: Signal<Vec<CommandItem>>,
    query: Binding<String>,
    visible_commands: Memo<Vec<CommandItem>>,
    selected_command: Memo<Option<CommandItem>>,
    selection: Selector<&'static str>,
    last_action: Signal<String>,
}

impl CommandPickerSurface {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);

        let commands = cx.signal(default_commands());
        let query = cx.binding(String::new());
        let selection = cx.selector(Some("open-workspace"));
        let commands_for_visible = commands.clone();
        let query_for_visible = query.clone();
        let visible_commands = cx.derived(move |cx| {
            let query = query_for_visible.get(cx);
            commands_for_visible.read(cx, |commands| visible_commands(commands, &query))
        });
        let selected_command =
            visible_commands.selected_by(cx, selection.clone(), |command| command.id);

        Self {
            commands,
            query,
            visible_commands,
            selected_command,
            selection,
            last_action: cx.signal("ready".to_string()),
        }
    }

    fn set_query(&self, cx: &mut App, query: impl Into<String>) {
        cx.batch(|cx| {
            self.query.set(cx, query.into());
            self.reconcile_selection(cx);
        });
    }

    fn select_next(&self, cx: &mut App) -> bool {
        self.selection.select_next(cx, self.visible_command_ids())
    }

    fn select_previous(&self, cx: &mut App) -> bool {
        self.selection
            .select_previous(cx, self.visible_command_ids())
    }

    fn execute_selected(&self, cx: &mut App) -> bool {
        let Some(command) = self.selected_command.get(cx) else {
            self.selection.clear(cx);
            return false;
        };

        self.last_action
            .set(cx, format!("ran {} ({})", command.title, command.group));
        true
    }

    #[cfg(test)]
    fn remove_command(&self, cx: &mut App, id: &'static str) {
        cx.batch(|cx| {
            self.commands.update(cx, |commands| {
                let before = commands.len();
                commands.retain(|command| command.id != id);
                before != commands.len()
            });
            self.reconcile_selection(cx);
        });
    }

    fn handle_key_down(&self, event: &KeyDownEvent, cx: &mut App) -> bool {
        match event.keystroke.key.as_str() {
            "arrow-down" => self.select_next(cx),
            "arrow-up" => self.select_previous(cx),
            "enter" => self.execute_selected(cx),
            _ => false,
        }
    }

    fn reconcile_selection(&self, cx: &mut App) {
        let ids = self.visible_command_ids();
        self.selection
            .reconcile_or_select_first(cx, ids.iter().copied());
    }

    fn visible_command_ids(&self) -> Vec<&'static str> {
        let query = self.query.signal().get_untracked();
        self.commands.peek(|commands| {
            commands
                .iter()
                .filter(|command| command_matches(command, &query))
                .map(|command| command.id)
                .collect()
        })
    }
}

impl ReactiveView for CommandPickerSurface {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let commands = self.visible_commands.get(cx);
        self.selection
            .reconcile_or_select_first_by(cx, &commands, |command| command.id);

        let selected = self.selection.get(cx);
        let query = self.query.get(cx);
        let last_action = self.last_action.get(cx);
        let selected_command = self.selected_command.get(cx);
        let selected_title = selected_command
            .as_ref()
            .map_or("None", |command| command.title);

        div()
            .id("command-picker")
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
                    .child(div().text_lg().child("Command picker"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Selected: {selected_title}")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Last action: {last_action}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        query_button("all", "All").on_click(cx.listener(|this, _, _, cx| {
                            this.set_query(cx, "");
                        })),
                    )
                    .child(query_button("query-file", "File").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.set_query(cx, "file");
                        },
                    )))
                    .child(query_button("query-branch", "Branch").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.set_query(cx, "branch");
                        },
                    )))
                    .child(query_button("previous", "Previous").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_previous(cx);
                        },
                    )))
                    .child(
                        query_button("next", "Next").on_click(cx.listener(|this, _, _, cx| {
                            this.select_next(cx);
                        })),
                    )
                    .child(
                        query_button("run", "Run").on_click(cx.listener(|this, _, _, cx| {
                            this.execute_selected(cx);
                        })),
                    ),
            )
            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(format!(
                "Query: {}",
                if query.is_empty() {
                    "all"
                } else {
                    query.as_str()
                }
            )))
            .child(
                div().flex().flex_col().gap_1().children(
                    commands
                        .into_iter()
                        .map(|command| command_row(command, selected)),
                ),
            )
            .into_any_element()
    }
}

impl Render for CommandPickerSurface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn default_commands() -> Vec<CommandItem> {
    vec![
        CommandItem::new(
            "open-workspace",
            "Open workspace",
            "File",
            "Switch to a project workspace",
        ),
        CommandItem::new(
            "save-file",
            "Save file",
            "File",
            "Persist the active buffer",
        ),
        CommandItem::new(
            "split-pane",
            "Split pane",
            "View",
            "Create another editor pane",
        ),
        CommandItem::new(
            "switch-branch",
            "Switch branch",
            "Git",
            "Checkout another working branch",
        ),
    ]
}

fn visible_commands(commands: &[CommandItem], query: &str) -> Vec<CommandItem> {
    commands
        .iter()
        .filter(|command| command_matches(command, query))
        .cloned()
        .collect()
}

fn command_matches(command: &CommandItem, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }

    command.title.to_ascii_lowercase().contains(&query)
        || command.group.to_ascii_lowercase().contains(&query)
        || command.description.to_ascii_lowercase().contains(&query)
}

fn query_button(id: &'static str, label: &'static str) -> Stateful<Div> {
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

fn command_row(command: CommandItem, selected: Option<&'static str>) -> impl IntoElement {
    let is_selected = selected == Some(command.id);

    div()
        .id(format!("command-row-{}", command.id))
        .px_3()
        .py_2()
        .rounded(px(6.0))
        .border_1()
        .border_color(if is_selected {
            rgb(0x60a5fa)
        } else {
            rgb(0x3f3f46)
        })
        .bg(if is_selected {
            rgb(0x1e3a8a)
        } else {
            rgb(0x27272a)
        })
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().truncate().child(command.title))
                .child(
                    div()
                        .truncate()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(command.description),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xcbd5e1))
                .child(command.group),
        )
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(620.0), px(360.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(CommandPickerSurface::new),
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
    use gpui::{Keystroke, TestApp};

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

    #[test]
    fn command_picker_navigation_uses_host_order() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            assert!(surface.select_next(cx));
        });

        let selected = app.update_entity(&root, |surface, _cx| surface.selection.get_untracked());
        assert_eq!(selected, Some("save-file"));
    }

    #[test]
    fn command_picker_filter_reconciles_selection_to_visible_command() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            surface.set_query(cx, "branch");
        });

        let (visible, selected) = app.update_entity(&root, |surface, cx| {
            (
                surface.visible_commands.read(cx, |commands| {
                    commands
                        .iter()
                        .map(|command| command.id)
                        .collect::<Vec<_>>()
                }),
                surface.selection.get_untracked(),
            )
        });

        assert_eq!(visible, vec!["switch-branch"]);
        assert_eq!(selected, Some("switch-branch"));
    }

    #[test]
    fn command_picker_selected_command_memo_tracks_selection_and_filter() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        let initial = app.update_entity(&root, |surface, cx| {
            surface.selected_command.get(cx).map(|command| command.id)
        });
        assert_eq!(initial, Some("open-workspace"));

        app.update_entity(&root, |surface, cx| {
            surface.select_next(cx);
        });
        let selected = app.update_entity(&root, |surface, cx| {
            surface.selected_command.get(cx).map(|command| command.id)
        });
        assert_eq!(selected, Some("save-file"));

        app.update_entity(&root, |surface, cx| {
            surface.set_query(cx, "branch");
        });
        let filtered = app.update_entity(&root, |surface, cx| {
            surface.selected_command.get(cx).map(|command| command.id)
        });
        assert_eq!(filtered, Some("switch-branch"));
    }

    #[test]
    fn command_picker_execute_selected_command_updates_host_action() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            surface.set_query(cx, "branch");
            assert!(surface.execute_selected(cx));
        });

        let last_action =
            app.update_entity(&root, |surface, _cx| surface.last_action.get_untracked());
        assert_eq!(last_action, "ran Switch branch (Git)");
    }

    #[test]
    fn command_picker_removal_reconciles_stale_selection() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            surface.remove_command(cx, "open-workspace");
        });

        let selected = app.update_entity(&root, |surface, _cx| surface.selection.get_untracked());
        assert_eq!(selected, Some("save-file"));
    }

    #[test]
    fn command_picker_keyboard_enter_runs_selected_command() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| CommandPickerSurface::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            assert!(surface.handle_key_down(&key("arrow-down"), cx));
            assert!(surface.handle_key_down(&key("enter"), cx));
        });

        let last_action =
            app.update_entity(&root, |surface, _cx| surface.last_action.get_untracked());
        assert_eq!(last_action, "ran Save file (File)");
    }
}
