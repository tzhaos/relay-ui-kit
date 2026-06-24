use gpui::{
    Anchor, AnyElement, AnyView, App, ClickEvent, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, WindowControlArea, div, point, px,
};
use relay::{
    KeyedSubViews, OrderedSelectionModel, ReactiveAppExt, ReactiveView, SelectionReconcilePolicy,
    Selector, Signal, SignalVecExt, use_ordered_selection_model, view::reactive_render,
};
use relay_uikit::patterns::{
    ActionsMenu, CommandMenu, CommandMenuItem, CommandMenuItemKind, CommandPalette, CommandRow,
    DiffView, FileKind, FileViewer, InputComposer, ItemPicker, KeybindingActionKind,
    KeybindingActions, KeybindingRow, KeybindingShortcut, KeybindingTable, MarkdownViewer,
    OutputLog, OutputSurface, PaneToolbar, PickerAction, PickerOption, Popover, QuickAction,
    SessionRow, SourceView, TabStrip, TabToolbar, TaskRow, TaskRowData, TopToolbar,
    WorkspaceBreadcrumb,
    display::KeyValue,
    layout::ListSection,
    navigation::{Tab, Tabs},
    output_resource_snapshot,
    overlay::{
        AnchoredOverlay, ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Select,
        SelectOption, TooltipBody,
    },
};
use relay_uikit::{
    ActiveTheme, Button, Divider, IconButton, IconName, Label, LabelSize, ListItem,
    ListItemSpacing, Pane, PaneSurface, PaneWidth, PanelHeader, SplitAxis, SplitPane, StatusBar,
    StatusDot, StatusItem, TextArea, Theme, ThemePreviewKind, Tone, WindowControls,
    interaction::{SelectionBinding, SelectionSource},
};

use super::{
    GalleryContentTab, GalleryScenesApp, GalleryState, PatternBranch, PatternCommand,
    PatternPreviewTab, PatternRowKind,
    shared::{scene_stack, section, strip},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let overlay_event_text = state.overlay_event.get(cx);
    let layout_body = layout_patterns(state, theme, cx);
    let display_body = display_patterns(state, theme);
    let navigation_body = navigation_patterns(state);
    let overlay_body = overlay_patterns(state, theme, &overlay_event_text, cx);
    let rows_body = row_patterns(state, cx);
    let tabs_body = tab_patterns(state, cx);
    let composer_body = composer_sample(state, host, cx);
    let output_body = output_patterns(state, host, theme, cx);
    let qa_body = quick_action_sample(state);
    let launcher_body = launcher_patterns(state);
    let command_picker_body = command_picker_patterns(state, theme, cx);
    let picker_body = picker_sample(state);
    let viewer_body = viewer_patterns(state, cx);

    let mut stack = scene_stack()
        .child(section(cx, "Layout patterns", layout_body))
        .child(section(cx, "Display patterns", display_body))
        .child(section(cx, "Navigation patterns", navigation_body));
    stack = stack.child(section(cx, "Overlay patterns", overlay_body));

    if state.pattern_dialog_open.get(cx) {
        stack = stack.child(settings_dialog(state));
    }
    if state.pattern_confirm_open.get(cx) {
        stack = stack.child(confirm_dialog(state));
    }
    stack = stack
        .child(section(cx, "Task Row & Session Row", rows_body))
        .child(section(cx, "Tab Strip & Toolbar", tabs_body))
        .child(section(cx, "Input Composer", composer_body))
        .child(section(cx, "Output Surface & Log", output_body))
        .child(section(cx, "Quick Actions", qa_body))
        .child(section(cx, "Command Menu & Keybindings", launcher_body))
        .child(section(cx, "Command Palette & Picker", command_picker_body))
        .child(section(cx, "Item Picker", picker_body))
        .child(section(cx, "File Viewer · Markdown · Diff", viewer_body));

    stack
}

// ── Composite pattern samples ─────────────────────────────────────────────

fn row_patterns(
    state: &GalleryState,
    _cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            TaskRow::new(
                "pat-task",
                TaskRowData {
                    title: "Implement relay patterns".into(),
                    status_label: "ACTIVE".into(),
                    status_tone: Tone::Accent,
                    branch: Some("relay/patterns".into()),
                    changed: 5,
                    review: 2,
                },
            )
            .selected_by(state.pattern_row_selection.clone(), PatternRowKind::Task),
        )
        .child(
            SessionRow::new("pat-session", "codex", "relay/patterns")
                .status(Tone::Accent)
                .active_by(state.pattern_row_selection.clone(), PatternRowKind::Session),
        )
}

fn tab_patterns(
    state: &GalleryState,
    _cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div().flex().flex_col().gap_2().child(
        TabToolbar::new()
            .tab(
                TabStrip::new("pat-tab1", "Terminal")
                    .active_by(
                        state.pattern_tab_selection.clone(),
                        PatternPreviewTab::Terminal,
                    )
                    .status(Tone::Accent),
            )
            .tab(
                TabStrip::new("pat-tab2", "Preview")
                    .active_by(
                        state.pattern_tab_selection.clone(),
                        PatternPreviewTab::Preview,
                    )
                    .status(Tone::Muted),
            )
            .tab(TabStrip::new("pat-tab3", "Review").active_by(
                state.pattern_tab_selection.clone(),
                PatternPreviewTab::Review,
            ))
            .actions(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(IconButton::new("pat-tab-search", IconName::Search))
                    .child(IconButton::new("pat-tab-more", IconName::Ellipsis)),
            ),
    )
}

fn composer_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    InputComposer::new(
        "patterns-composer",
        TextArea::bound(
            "patterns-composer-input",
            state.composer_focus.clone(),
            state.composer_input.clone(),
        )
        .placeholder("Ask Codex to productize the next component batch...")
        .min_rows(4)
        .bordered(false),
    )
    .leading(
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Label::new("Ctrl+Enter submits").size(LabelSize::Small))
            .child(KeybindingShortcut::new(["Ctrl", "Enter"])),
    )
    .trailing(
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Button::new("patterns-composer-attach", "Context").ghost())
            .child(
                Button::new("patterns-composer-send", "Send")
                    .primary()
                    .icon(IconName::ArrowRight)
                    .on_click({
                        let host = host.clone();
                        let composer_input = state.composer_input.clone();
                        move |_event, _window, cx| {
                            let summary = composer_input.get(cx).value().trim().to_string();
                            if !summary.is_empty() {
                                host.update(cx, |this, cx| {
                                    this.add_feedback_toast(
                                        cx,
                                        format!("Composer submitted: {}", summary),
                                    );
                                });
                            }
                            composer_input.update(cx, |state| {
                                state.clear();
                                true
                            });
                        }
                    }),
            ),
    )
    .footer(
        div().text_xs().text_color(cx.theme().text_muted).child(
            "InputComposer now wraps the real multiline editor instead of a placeholder shell.",
        ),
    )
    .floating(true)
}

fn output_patterns(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let connected = state.core_disclosure_open.get(cx);
    let output = output_resource_snapshot(
        &state.pattern_output,
        cx,
        "Loading output",
        "Refreshing output",
        |line_count| format!("{line_count} lines ready"),
        "Refresh failed",
        |error| format!("refresh failed: {error}"),
    );
    let loading = output.loading;
    let status_text = output.status_text;
    let lines = output.lines;
    let refresh_host = host.clone();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            OutputSurface::new("pat-output", OutputLog::new(lines).prompt("> "))
                .connected(connected),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new(
                        "pat-output-refresh",
                        if loading {
                            "Refreshing"
                        } else {
                            "Refresh output"
                        },
                    )
                    .icon(IconName::RefreshCw)
                    .ghost()
                    .disabled(loading)
                    .on_click(move |_event, _window, cx| {
                        refresh_host.update(cx, |this, cx| {
                            this.reload_pattern_output(cx);
                        });
                    }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("{status_text} - connected: {connected}")),
                ),
        )
}

fn quick_action_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div()
        .flex()
        .gap_2()
        .flex_wrap()
        .child(
            QuickAction::new("qa-codex", "Launch Codex", "codex").on_click({
                let log = log.clone();
                move |_, _, cx: &mut App| {
                    log.set(cx, "QuickAction: Launch Codex".into());
                }
            }),
        )
        .child(
            QuickAction::new("qa-build", "Build", "cargo build").on_click({
                move |_, _, cx: &mut App| {
                    log.set(cx, "QuickAction: Build".into());
                }
            }),
        )
}

fn launcher_patterns(state: &GalleryState) -> impl IntoElement {
    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    CommandMenu::new(
                        "patterns-command-menu",
                        vec![
                            CommandMenuItem::new(
                                PatternCommand::NewTerminal,
                                "New terminal",
                                IconName::Terminal,
                            )
                            .detail("Open a shell in the current worktree")
                            .kind(CommandMenuItemKind::Terminal),
                            CommandMenuItem::new(
                                PatternCommand::LaunchAgent,
                                "Launch agent",
                                IconName::Bot,
                            )
                            .detail("Run Codex against the selected relay surface")
                            .kind(CommandMenuItemKind::Agent),
                            CommandMenuItem::new(
                                PatternCommand::OpenReview,
                                "Open review",
                                IconName::MessageSquareText,
                            )
                            .detail("Jump to the review context panel")
                            .kind(CommandMenuItemKind::Action),
                        ],
                    )
                    .on_select(pattern_command_event(state)),
                )
                .child(
                    div()
                        .w(px(232.0))
                        .child(ActionsMenu::new("patterns-actions-menu").on_select({
                            let log = state.overlay_event.clone();
                            move |action, _window, cx| {
                                log.set(cx, format!("Branch actions menu: {}", action.label()));
                            }
                        })),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(KeybindingTable::new(vec![
                    KeybindingRow::new("Open command palette")
                        .description("Focus the launcher without leaving the current pane")
                        .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
                        .action(
                            KeybindingActions::new("patterns-keybinding-palette")
                                .on_edit(pattern_keybinding_action(
                                    state,
                                    KeybindingActionKind::Edit,
                                    "Open command palette",
                                ))
                                .on_reset(pattern_keybinding_action(
                                    state,
                                    KeybindingActionKind::Reset,
                                    "Open command palette",
                                )),
                        ),
                    KeybindingRow::new("Open review")
                        .description("Jump into the review queue surface")
                        .shortcut(KeybindingShortcut::new(["Ctrl", "R"]))
                        .action(
                            KeybindingActions::new("patterns-keybinding-review")
                                .on_edit(pattern_keybinding_action(
                                    state,
                                    KeybindingActionKind::Edit,
                                    "Open review",
                                ))
                                .on_clear(pattern_keybinding_action(
                                    state,
                                    KeybindingActionKind::Clear,
                                    "Open review",
                                )),
                        ),
                ]))
                .child(
                    div()
                        .text_xs()
                        .text_color(gpui::rgb(0x7a8291))
                        .child("KeybindingActions now expose keyboard focus and activation semantics alongside pointer clicks."),
                ),
        )
}

fn command_picker_patterns(
    state: &GalleryState,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let selected_command = state
        .pattern_command_selection
        .get(cx)
        .map(PatternCommand::label)
        .unwrap_or("None");
    let selected_branch = state
        .pattern_branch_selection
        .get(cx)
        .map(PatternBranch::label)
        .unwrap_or("None");

    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(command_palette_sample(state))
                .child(KeyValue::new("Selected command", selected_command)),
        )
        .child(
            div()
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .p_3()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div().w(px(320.0)).child(
                        ItemPicker::new(
                            "patterns-branch-selector",
                            PatternBranch::Main,
                            pattern_branch_options(),
                        )
                        .selected_with(SelectionSource::ordered_selection_model(
                            state.pattern_branch_selection.clone(),
                        ))
                        .open_bound(state.command_popover_open.clone())
                        .actions(pattern_branch_actions())
                        .on_select({
                            let open = state.command_popover_open.clone();
                            let log = state.overlay_event.clone();
                            move |key, _window, cx| {
                                open.set(cx, false);
                                log.set(cx, format!("Branch selected: {}", key.label()));
                            }
                        })
                        .on_action({
                            let log = state.overlay_event.clone();
                            move |key, _window, cx| {
                                log.set(cx, format!("Branch action: {key}"));
                            }
                        }),
                    ),
                )
                .child(KeyValue::new("Selected branch", selected_branch)),
        )
}

fn command_palette_sample(state: &GalleryState) -> impl IntoElement + use<> {
    let selection = state.pattern_command_selection.clone();

    CommandPalette::new("Command router")
        .row(
            CommandRow::new(
                "patterns-command-terminal",
                PatternCommand::NewTerminal,
                "New terminal",
            )
            .detail("Open a shell in the selected branch")
            .icon(IconName::Terminal)
            .shortcut(KeybindingShortcut::new(["Ctrl", "`"]))
            .selected_with(SelectionBinding::ordered_selection_model(
                selection.clone(),
                PatternCommand::NewTerminal,
            ))
            .on_select(pattern_command_event(state)),
        )
        .row(
            CommandRow::new(
                "patterns-command-agent",
                PatternCommand::LaunchAgent,
                "Launch agent",
            )
            .detail("Start a Codex session for the workspace")
            .icon(IconName::Bot)
            .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
            .selected_with(SelectionBinding::ordered_selection_model(
                selection.clone(),
                PatternCommand::LaunchAgent,
            ))
            .on_select(pattern_command_event(state)),
        )
        .row(
            CommandRow::new(
                "patterns-command-review",
                PatternCommand::OpenReview,
                "Open review",
            )
            .detail("Focus the review panel")
            .icon(IconName::MessageSquareText)
            .shortcut(KeybindingShortcut::new(["Ctrl", "R"]))
            .selected_with(SelectionBinding::ordered_selection_model(
                selection,
                PatternCommand::OpenReview,
            ))
            .on_select(pattern_command_event(state)),
        )
}

fn pattern_command_event(
    state: &GalleryState,
) -> impl Fn(PatternCommand, &mut Window, &mut App) + 'static {
    let log = state.overlay_event.clone();
    move |key, _window, cx| {
        log.set(cx, format!("Command selected: {}", key.label()));
    }
}

fn pattern_branch_options() -> Vec<PickerOption<PatternBranch>> {
    vec![
        PickerOption::new(PatternBranch::Main, PatternBranch::Main.label())
            .detail(PatternBranch::Main.detail()),
        PickerOption::new(
            PatternBranch::RelayRuntime,
            PatternBranch::RelayRuntime.label(),
        )
        .detail(PatternBranch::RelayRuntime.detail()),
        PickerOption::new(
            PatternBranch::GalleryPatterns,
            PatternBranch::GalleryPatterns.label(),
        )
        .detail(PatternBranch::GalleryPatterns.detail()),
    ]
}

fn pattern_branch_actions() -> Vec<PickerAction> {
    vec![
        PickerAction::new("branch:create", "Create branch", IconName::Plus),
        PickerAction::new("branch:sync", "Sync branches", IconName::RefreshCw),
    ]
}

fn picker_sample(state: &GalleryState) -> AnyElement {
    cached_project_picker(state.pattern_project_picker.clone())
}

fn cached_project_picker(picker: Entity<PatternProjectPicker>) -> AnyElement {
    let view: AnyView = picker.into();
    view.cached(gpui::StyleRefinement::default().w_full())
        .into_any_element()
}

#[derive(Clone, PartialEq, Eq)]
struct PatternProject {
    id: u64,
    label: String,
    detail: String,
    tone: Tone,
    status: &'static str,
}

impl PatternProject {
    fn new(
        id: u64,
        label: impl Into<String>,
        detail: impl Into<String>,
        tone: Tone,
        status: &'static str,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            detail: detail.into(),
            tone,
            status,
        }
    }
}

pub(super) struct PatternProjectPicker {
    projects: Signal<Vec<PatternProject>>,
    rows: KeyedSubViews<u64, PatternProjectRow>,
    selection: OrderedSelectionModel<u64>,
    next_id: u64,
    revision: u64,
}

impl PatternProjectPicker {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        let projects = cx.signal(vec![
            PatternProject::new(
                1,
                "relay-ui-kit",
                "workspace / main",
                Tone::Accent,
                "ACTIVE",
            ),
            PatternProject::new(2, "relay", "crates / runtime", Tone::Info, "READY"),
            PatternProject::new(3, "gallery", "bin / relay_gallery", Tone::Warning, "REVIEW"),
        ]);
        let projects_for_selection = projects.clone();
        let selection = use_ordered_selection_model(
            cx,
            Some(1),
            move |cx| {
                projects_for_selection.read(cx, |projects| {
                    projects.iter().map(|project| project.id).collect()
                })
            },
            SelectionReconcilePolicy::SelectFirst,
        );

        Self {
            projects,
            rows: KeyedSubViews::new(),
            selection,
            next_id: 4,
            revision: 0,
        }
    }

    fn cycle_selection(&self, cx: &mut App) {
        let _ = self.selection.select_next(cx);
    }

    fn rotate_projects(&self, cx: &mut App) {
        self.projects.update(cx, |projects| {
            if projects.len() < 2 {
                return false;
            }
            projects.rotate_left(1);
            true
        });
    }

    fn rename_selected(&mut self, cx: &mut App) {
        let Some(selected) = self.selection.get_untracked() else {
            return;
        };
        self.revision = self.revision.wrapping_add(1);
        let revision = self.revision;

        self.projects.update(cx, |projects| {
            let Some(project) = projects.iter_mut().find(|project| project.id == selected) else {
                return false;
            };
            project.label = format!("{} r{revision}", project.label);
            project.detail = format!("workspace / refresh {revision:02}");
            true
        });
    }

    fn add_project(&mut self, cx: &mut App) {
        let id = self.next_id;
        self.next_id += 1;
        self.projects.push_selected_by(
            cx,
            self.selection.selection().selector(),
            PatternProject::new(
                id,
                format!("generated-{id:02}"),
                "workspace / generated",
                Tone::Secondary,
                "NEW",
            ),
            |project| project.id,
        );
    }
}

impl ReactiveView for PatternProjectPicker {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let projects = self.projects.get(cx);

        let selection = self.selection.selection().selector().clone();
        self.rows.sync_with_selector(
            cx,
            self.selection.selection().selector(),
            projects,
            |project| project.id,
            move |project, _cx| PatternProjectRow::new(project, selection.clone()),
            |project, row, _cx| row.update_project(project),
        );

        div()
            .w(px(560.0))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("pattern-project-cycle", "Cycle")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.cycle_selection(cx);
                            })),
                    )
                    .child(
                        Button::new("pattern-project-rotate", "Rotate")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.rotate_projects(cx);
                            })),
                    )
                    .child(
                        Button::new("pattern-project-rename", "Rename Active")
                            .ghost()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.rename_selected(cx);
                            })),
                    )
                    .child(Button::new("pattern-project-add", "Add").ghost().on_click(
                        cx.listener(|this, _event, _window, cx| {
                            this.add_project(cx);
                        }),
                    )),
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

impl Render for PatternProjectPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct PatternProjectRow {
    project: PatternProject,
    selection: Selector<u64>,
}

impl PatternProjectRow {
    fn new(project: &PatternProject, selection: Selector<u64>) -> Self {
        Self {
            project: project.clone(),
            selection,
        }
    }

    fn update_project(&mut self, project: &PatternProject) -> bool {
        if self.project == *project {
            false
        } else {
            self.project = project.clone();
            true
        }
    }
}

impl ReactiveView for PatternProjectRow {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let project = &self.project;
        let selected = self.selection.is_selected(cx, project.id);
        let selection = self.selection.clone();
        let id = project.id;

        ListItem::new(format!("pattern-project-{}", project.id))
            .spacing(ListItemSpacing::Relaxed)
            .selected(selected)
            .start_slot(StatusDot::new(project.tone))
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
                            .child(project.label.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(project.detail.clone()),
                    ),
            )
            .end_slot(
                div()
                    .text_size(px(11.0))
                    .text_color(project.tone.fg(&theme))
                    .child(project.status),
            )
            .on_click(move |_event, _window, cx| {
                selection.select(cx, id);
            })
            .into_any_element()
    }
}

impl Render for PatternProjectRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn viewer_patterns(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let active_tab = state.content_tab.get(cx);
    let active_label = active_tab.label().to_string();
    let theme = *cx.theme();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_xs().text_color(theme.text_muted).child(format!(
            "Shared with the navigation tabs above: {active_label}"
        )))
        .child(
            div().h(px(280.0)).child(match active_tab {
                GalleryContentTab::Files => FileViewer::new(
                    "crates/relay_uikit/src/patterns/source_view.rs",
                    FileKind::Code,
                    SourceView::new(VIEWER_SAMPLE).language("rust"),
                )
                .detail("Source preview"),
                GalleryContentTab::Diff => FileViewer::new(
                    "crates/relay_uikit/src/patterns/diff_view.rs",
                    FileKind::Diff,
                    DiffView::from_text_diff(VIEWER_DIFF_BEFORE, VIEWER_DIFF_AFTER),
                )
                .detail("Unified diff"),
                GalleryContentTab::Review => FileViewer::new(
                    "docs/relay-uikit-guidelines.md",
                    FileKind::Markdown,
                    MarkdownViewer::new(VIEWER_REVIEW_MARKDOWN),
                )
                .detail("Review context"),
            }),
        )
}

const VIEWER_SAMPLE: &str = "pub struct OutputLine {\n    text: String,\n    style: OutputLineStyle,\n}\n\nimpl OutputLine {\n    pub fn new(text: impl Into<String>) -> Self {\n        Self { text: text.into(), style: OutputLineStyle::Output }\n    }\n}";
const VIEWER_DIFF_BEFORE: &str =
    "pub fn launch_agent() {\n    enqueue(\"relay\");\n    log(\"pending\");\n}\n";
const VIEWER_DIFF_AFTER: &str = "pub fn launch_agent() {\n    enqueue(\"relay_v2\");\n    log(\"ready\");\n    notify(\"agent started\");\n}\n";
const VIEWER_REVIEW_MARKDOWN: &str = "# Relay UIKit Review\n\n- Productize every exported component.\n- Keep focus, keyboard, and pointer behavior aligned.\n- Remove compatibility aliases instead of preserving them.\n\n> Gallery scenes should behave like real desktop surfaces.";

fn layout_patterns(
    state: &GalleryState,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let split_status = {
        let split_state = state.pattern_vertical_split.read(cx);
        if split_state.is_resizing() {
            format!("Vertical split preview: {:.0}px", split_state.first_size())
        } else {
            format!(
                "Vertical split committed: {:.0}px",
                split_state.committed_first_size()
            )
        }
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .h(px(38.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.chrome)
                .px_2()
                .child(
                    TopToolbar::new()
                        .leading(Label::new("TopToolbar").strong())
                        .center(WorkspaceBreadcrumb::new(vec![
                            "Relay".into(),
                            "Patterns".into(),
                            "Shell".into(),
                        ]))
                        .trailing(
                            PaneToolbar::new()
                                .action(IconButton::new("toolbar-search", IconName::Search))
                                .action(IconButton::new("toolbar-refresh", IconName::RefreshCw))
                                .action(IconButton::new("toolbar-more", IconName::Ellipsis)),
                        ),
                ),
        )
        .child(
            div()
                .h(px(360.0))
                .rounded(px(10.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.app_bg)
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(34.0))
                        .border_b_1()
                        .border_color(theme.border)
                        .bg(theme.chrome)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .h_full()
                                .min_w_0()
                                .flex_1()
                                .px_3()
                                .flex()
                                .items_center()
                                .gap_3()
                                .window_control_area(WindowControlArea::Drag)
                                .child(Label::new("Relay workbench shell").strong())
                                .child(
                                    div()
                                        .min_w_0()
                                        .text_xs()
                                        .truncate()
                                        .text_color(theme.text_muted)
                                        .child(
                                            "Direct Pane, SplitPane, PaneSurface, and WindowControls composition",
                                        ),
                                ),
                        )
                        .child(WindowControls::new()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .child(
                            Pane::new(
                                PaneWidth::Fixed(196.0),
                                div()
                                    .p_2()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        ListItem::new("patterns-shell-nav-agent")
                                            .selected(true)
                                            .start_slot(StatusDot::new(Tone::Accent))
                                            .child("Agent work")
                                            .end_slot(Label::new("LIVE").size(LabelSize::Small)),
                                    )
                                    .child(
                                        ListItem::new("patterns-shell-nav-gallery")
                                            .start_slot(StatusDot::new(Tone::Info))
                                            .child("Gallery migration"),
                                    )
                                    .child(
                                        ListItem::new("patterns-shell-nav-cleanup")
                                            .start_slot(StatusDot::new(Tone::Warning))
                                            .child("Cleanup backlog"),
                                    ),
                            )
                            .surface(PaneSurface::Chrome)
                            .header(PanelHeader::new("Navigator").icon(IconName::LayoutGrid)),
                        )
                        .child(Divider::vertical())
                        .child(
                            Pane::new(
                                PaneWidth::Flex,
                                SplitPane::new(
                                    "patterns-shell-vertical-split",
                                    Pane::new(
                                        PaneWidth::Flex,
                                        div()
                                            .p_4()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(Label::new("Preview region").strong())
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme.text_secondary)
                                                    .child(
                                                        "Inset surfaces are for focused content where the parent shell should still frame the task.",
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .rounded(px(8.0))
                                                    .border_1()
                                                    .border_color(theme.border)
                                                    .bg(theme.panel)
                                                    .p_3()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(
                                                        "Drag the split handle to validate host-owned resize state and keyboard-accessible handle behavior.",
                                                    ),
                                            ),
                                    )
                                    .surface(PaneSurface::Inset),
                                    Pane::new(
                                        PaneWidth::Flex,
                                        div()
                                            .p_3()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(Label::new("Console strip").strong())
                                            .child(
                                                div()
                                                    .rounded(px(8.0))
                                                    .border_1()
                                                    .border_color(theme.border)
                                                    .bg(theme.chrome)
                                                    .p_3()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(
                                                        "Transparent pane surfaces let the child decide how much chrome it needs while inheriting the shell background.",
                                                    ),
                                            ),
                                    )
                                    .surface(PaneSurface::Transparent),
                                )
                                .axis(SplitAxis::Vertical)
                                .state(state.pattern_vertical_split.clone())
                                .min_sizes(108.0, 120.0)
                                .on_resize({
                                    let overlay_event = state.overlay_event.clone();
                                    move |size, _window, cx| {
                                        overlay_event.set(
                                            cx,
                                            format!("Workbench split preview: {:.0}px", size),
                                        );
                                    }
                                })
                                .on_resize_end({
                                    let overlay_event = state.overlay_event.clone();
                                    move |_window, cx| {
                                        overlay_event
                                            .set(cx, "Workbench split committed".into());
                                    }
                                }),
                            )
                            .surface(PaneSurface::Panel)
                            .header(
                                PanelHeader::new("Workbench")
                                    .icon(IconName::Terminal)
                                    .trailing(
                                        PaneToolbar::new()
                                            .action(
                                                IconButton::new(
                                                    "patterns-shell-search",
                                                    IconName::Search,
                                                )
                                                .aria_label("Search workbench"),
                                            )
                                            .action(
                                                IconButton::new(
                                                    "patterns-shell-refresh",
                                                    IconName::RefreshCw,
                                                )
                                                .aria_label("Refresh workbench"),
                                            )
                                            .action(
                                                IconButton::new(
                                                    "patterns-shell-more",
                                                    IconName::Ellipsis,
                                                )
                                                .aria_label("Open workbench actions"),
                                            ),
                                    ),
                            ),
                        )
                        .child(Divider::vertical())
                        .child(
                            Pane::new(
                                PaneWidth::Fixed(236.0),
                                div()
                                    .p_3()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(KeyValue::new("Branch", "relay_v2"))
                                    .child(KeyValue::new("Layer", "patterns/layout"))
                                    .child(KeyValue::new("Surface", "Inset inspector"))
                                    .child(KeyValue::new("State", "host-owned split")),
                            )
                            .surface(PaneSurface::Inset)
                            .header(PanelHeader::new("Inspector").icon(IconName::Settings)),
                        ),
                )
                .child(
                    StatusBar::new()
                        .left(
                            StatusItem::new("Surface", "Workbench")
                                .icon(IconName::Terminal)
                                .tone(Tone::Info),
                        )
                        .left(StatusItem::new("Split", "Vertical").tone(Tone::Secondary))
                        .right(StatusItem::new("Resize", split_status.clone()).tone(Tone::Accent)),
                ),
        )
        .child(
            strip()
                .child(
                    Button::new("layout-split-compact", "Compact console")
                        .icon(IconName::Terminal)
                        .on_click({
                            let split_state = state.pattern_vertical_split.clone();
                            let overlay_event = state.overlay_event.clone();
                            move |_event, _window, cx| {
                                split_state.update(cx, |state, _cx| {
                                    state.set_first_size(112.0);
                                });
                                overlay_event.set(
                                    cx,
                                    "Workbench split reset to editor-heavy layout".into(),
                                );
                            }
                        }),
                )
                .child(
                    Button::new("layout-split-balance", "Balance split")
                        .icon(IconName::RefreshCw)
                        .on_click({
                            let split_state = state.pattern_vertical_split.clone();
                            let overlay_event = state.overlay_event.clone();
                            move |_event, _window, cx| {
                                split_state.update(cx, |state, _cx| {
                                    state.set_first_size(164.0);
                                });
                                overlay_event
                                    .set(cx, "Workbench split balanced for review".into());
                            }
                        }),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format!(
                    "PaneWidth, PaneSurface, SplitAxis, and WindowControls now land together in one app-like shell instead of isolated snippets. {split_status}"
                )),
        )
}

fn display_patterns(state: &GalleryState, theme: Theme) -> impl IntoElement {
    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(
            ListSection::new("Session metadata")
                .count(4)
                .trailing(
                    Button::new("metadata-copy", "Copy")
                        .ghost()
                        .on_click(pattern_event(state, "Metadata copied")),
                )
                .child(
                    div()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.panel)
                        .p_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(KeyValue::new("Project", "relay-ui-kit").icon(IconName::Folder))
                        .child(KeyValue::new("Branch", "master").icon(IconName::GitBranch))
                        .child(KeyValue::new("Layer", "patterns/display"))
                        .child(KeyValue::new("State", "host-owned")),
                ),
        )
        .child(
            div()
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(Label::new("TooltipBody").strong())
                .child(TooltipBody::new(
                    "Hover and focus labels use this compact body",
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Display patterns are composed but still product-neutral."),
                ),
        )
}

fn navigation_patterns(state: &GalleryState) -> impl IntoElement {
    div().max_w(px(640.0)).child(Tabs::bound(
        "patterns-navigation-tabs",
        vec![
            Tab::new(GalleryContentTab::Files, "Files").icon(IconName::FileText),
            Tab::new(GalleryContentTab::Diff, "Diff")
                .icon(IconName::FileDiff)
                .count(12),
            Tab::new(GalleryContentTab::Review, "Review")
                .icon(IconName::MessageSquareText)
                .count(3),
        ],
        state.content_tab.clone(),
    ))
}

fn overlay_patterns(
    state: &GalleryState,
    theme: Theme,
    overlay_event_text: &str,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div()
        .relative()
        .min_h(px(236.0))
        .flex()
        .items_start()
        .gap_4()
        .flex_wrap()
        .child(div().w(px(260.0)).child(Select::bound(
            "patterns-overlay-select",
            state.theme_choice.clone(),
            vec![
                SelectOption::new(ThemePreviewKind::System, "System")
                    .detail("Follow OS appearance"),
                SelectOption::new(ThemePreviewKind::Light, "Light"),
                SelectOption::new(ThemePreviewKind::Dark, "Dark"),
            ],
        )))
        .child(
            div().w(px(220.0)).child(
                DropdownMenu::new(
                    "patterns-dropdown",
                    Button::new("patterns-dropdown-btn", "Dropdown Menu")
                        .variant(relay_uikit::ButtonVariant::Secondary)
                        .icon(IconName::ChevronDown)
                        .on_click({
                            let open = state.command_context_open.clone();
                            move |_event, _window, cx| {
                                open.update(cx, |v| {
                                    *v = !*v;
                                    true
                                });
                            }
                        }),
                    vec![
                        MenuItem::header("Actions"),
                        MenuItem::new("New file").icon(IconName::Plus),
                        MenuItem::new("Duplicate").icon(IconName::FileText),
                        MenuItem::separator(),
                        MenuItem::new("Delete").icon(IconName::Archive).danger(),
                    ],
                )
                .open(state.command_context_open.get(cx))
                .min_width(200.0)
                .on_dismiss({
                    let open = state.command_context_open.clone();
                    move |_window, cx| {
                        open.set(cx, false);
                    }
                }),
            ),
        )
        .child(
            div().w(px(240.0)).child(
                AnchoredOverlay::new(
                    "patterns-anchored-menu",
                    Button::new("patterns-anchored-menu-trigger", "Anchored Menu")
                        .variant(relay_uikit::ButtonVariant::Secondary)
                        .icon(IconName::ChevronDown)
                        .on_click({
                            let open = state.pattern_anchor_open.clone();
                            move |_event, _window, cx| {
                                open.update(cx, |value| {
                                    *value = !*value;
                                    true
                                });
                            }
                        }),
                    Menu::new(
                        "patterns-direct-menu",
                        vec![
                            MenuItem::header("Review lanes"),
                            anchored_menu_action(
                                state,
                                "Review in current split",
                                IconName::Terminal,
                            )
                            .detail("Keep the current worktree context")
                            .checked(true),
                            MenuItem::new("Assign lane")
                                .icon(IconName::LayoutGrid)
                                .submenu_items(vec![
                                    anchored_menu_action(
                                        state,
                                        "Assign lane: agent",
                                        IconName::Bot,
                                    ),
                                    anchored_menu_action(
                                        state,
                                        "Assign lane: gallery",
                                        IconName::FileText,
                                    ),
                                ]),
                            MenuItem::separator(),
                            anchored_menu_action(state, "Archive review slice", IconName::Archive)
                                .danger(),
                        ],
                    )
                    .min_width(228.0),
                )
                .open(state.pattern_anchor_open.get(cx))
                .anchor(Anchor::TopLeft)
                .attach(Anchor::BottomLeft)
                .offset(point(px(0.0), px(6.0)))
                .full_width(true)
                .on_dismiss({
                    let open = state.pattern_anchor_open.clone();
                    let overlay_event = state.overlay_event.clone();
                    move |_window, cx| {
                        open.set(cx, false);
                        overlay_event.set(cx, "Anchored overlay dismissed".into());
                    }
                }),
            ),
        )
        .child(
            Button::new("patterns-dialog-open", "Open Dialog")
                .icon(IconName::Settings)
                .on_click({
                    let dialog_open = state.pattern_dialog_open.clone();
                    move |_event, _window, cx| {
                        dialog_open.set(cx, true);
                    }
                }),
        )
        .child(
            Button::new("patterns-confirm-open", "Open Confirm")
                .icon(IconName::Archive)
                .on_click({
                    let confirm_open = state.pattern_confirm_open.clone();
                    move |_event, _window, cx| {
                        confirm_open.set(cx, true);
                    }
                }),
        )
        .child(
            div()
                .relative()
                .w(px(240.0))
                .h(px(148.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .child(
                    div()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(Label::new("ContextMenu").strong())
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Static overlay sample"),
                        ),
                )
                .child(
                    ContextMenu::new(
                        "patterns-context-menu",
                        vec![
                            MenuItem::header("Terminal"),
                            menu_action(state, "Split right", IconName::ArrowRight),
                            menu_action(state, "Rename", IconName::Settings),
                            MenuItem::separator(),
                            menu_action(state, "Close", IconName::Archive).danger(),
                        ],
                    )
                    .offset(24.0, 64.0)
                    .min_width(190.0),
                ),
        )
        .child(
            Popover::new("patterns-inline-popover")
                .title("Popover")
                .icon(IconName::MessageSquareText)
                .width(260.0)
                .child(div().text_sm().text_color(theme.text_secondary).child(
                    "Use this for compact inline detail, not for full-screen workflow pivots.",
                ))
                .footer(div().text_xs().text_color(theme.text_muted).child(
                    "Popover remains lightweight and anchored to the current task context.",
                )),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format!("Pattern event: {}", overlay_event_text)),
        )
}

fn settings_dialog(state: &GalleryState) -> impl IntoElement {
    Dialog::new("patterns-settings-dialog", "Pattern settings")
        .description("Dialog uses the overlay layer and host-owned dismiss state.")
        .icon(IconName::Settings)
        .width(420.0)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(KeyValue::new("Layer", "patterns/overlay"))
                .child(KeyValue::new("Dismiss", "backdrop or button"))
                .child(KeyValue::new("Motion", "fade-slide")),
        )
        .footer(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new("patterns-dialog-cancel", "Cancel")
                        .ghost()
                        .on_click(close_dialog(state, "Dialog cancelled")),
                )
                .child(
                    Button::new("patterns-dialog-save", "Save")
                        .primary()
                        .on_click(close_dialog(state, "Dialog saved")),
                ),
        )
        .on_dismiss(close_dialog(state, "Dialog dismissed"))
}

fn confirm_dialog(state: &GalleryState) -> impl IntoElement {
    ConfirmDialog::new(
        "patterns-confirm-dialog",
        "Archive generated worktree",
        "This removes the active preview worktree from the gallery workspace.",
    )
    .confirm_label("Archive")
    .danger(true)
    .on_confirm(close_confirm_dialog(
        state,
        "Confirm dialog saved and archived the worktree",
    ))
    .on_dismiss(close_confirm_dialog(state, "Confirm dialog dismissed"))
    .on_cancel(close_confirm_dialog(state, "Confirm dialog cancelled"))
}

fn menu_action(state: &GalleryState, label: &'static str, icon: IconName) -> MenuItem {
    MenuItem::new(label)
        .icon(icon)
        .on_click(pattern_event(state, label))
}

fn anchored_menu_action(state: &GalleryState, label: &'static str, icon: IconName) -> MenuItem {
    let menu_open = state.pattern_anchor_open.clone();
    let overlay_event = state.overlay_event.clone();

    MenuItem::new(label)
        .icon(icon)
        .on_click(move |_event, _window, cx| {
            menu_open.set(cx, false);
            overlay_event.set(cx, format!("Anchored menu: {label}"));
        })
}

fn pattern_event(
    state: &GalleryState,
    message: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        overlay_event.set(cx, message.to_string());
    }
}

fn pattern_keybinding_action(
    state: &GalleryState,
    action: KeybindingActionKind,
    command: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        overlay_event.set(cx, format!("Keybinding {}: {command}", action.label()));
    }
}

fn close_dialog(
    state: &GalleryState,
    message: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let dialog_open = state.pattern_dialog_open.clone();
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        dialog_open.set(cx, false);
        overlay_event.set(cx, message.to_string());
    }
}

fn close_confirm_dialog(
    state: &GalleryState,
    message: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let confirm_open = state.pattern_confirm_open.clone();
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        confirm_open.set(cx, false);
        overlay_event.set(cx, message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use gpui::{EntityId, TestApp};

    use super::super::GallerySurface;
    use super::*;

    fn row_ids(rows: &KeyedSubViews<u64, PatternProjectRow>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    #[test]
    fn project_picker_reuses_rows_when_projects_rotate() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            PatternProjectPicker::new(cx)
        });
        let root = window.root();

        window.draw();
        let initial = app.update_entity(&root, |picker, _cx| row_ids(&picker.rows));

        app.update_entity(&root, |picker, cx| {
            picker.rotate_projects(cx);
        });
        window.draw();

        let rotated = app.update_entity(&root, |picker, _cx| row_ids(&picker.rows));
        assert_eq!(
            rotated,
            vec![(2, initial[1].1), (3, initial[2].1), (1, initial[0].1)]
        );
    }

    #[test]
    fn project_picker_reuses_rows_when_selected_project_updates() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            PatternProjectPicker::new(cx)
        });
        let root = window.root();

        window.draw();
        let initial = app.update_entity(&root, |picker, _cx| row_ids(&picker.rows));

        app.update_entity(&root, |picker, cx| {
            picker.rename_selected(cx);
        });
        window.draw();

        let updated = app.update_entity(&root, |picker, _cx| row_ids(&picker.rows));
        assert_eq!(updated, initial);
    }

    #[test]
    fn project_picker_remove_active_reselects_first_remaining_project() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            PatternProjectPicker::new(cx)
        });
        let root = window.root();

        window.draw();
        app.update_entity(&root, |picker, cx| {
            picker.projects.remove_selected_by(
                cx,
                picker.selection.selection().selector(),
                |project| project.id,
            );
        });
        window.draw();

        let (selected, projects) = app.update_entity(&root, |picker, _cx| {
            (
                picker.selection.get_untracked(),
                picker
                    .projects
                    .get_untracked()
                    .into_iter()
                    .map(|project| project.id)
                    .collect::<Vec<_>>(),
            )
        });
        assert_eq!(selected, Some(2));
        assert_eq!(projects, vec![2, 3]);
    }

    #[test]
    fn command_picker_selection_models_drive_gallery_labels() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            GalleryScenesApp::new(cx)
        });
        let root = window.root();

        app.update_entity(&root, |gallery, cx| {
            gallery.set_surface(GallerySurface::Patterns, cx);
        });
        window.draw();

        let initial = app.update_entity(&root, |gallery, cx| {
            (
                gallery
                    .state
                    .pattern_command_selection
                    .get(cx)
                    .map(PatternCommand::label)
                    .unwrap_or("None"),
                gallery
                    .state
                    .pattern_branch_selection
                    .get(cx)
                    .map(PatternBranch::label)
                    .unwrap_or("None"),
            )
        });
        assert_eq!(initial, ("New terminal", "main"));

        app.update_entity(&root, |gallery, cx| {
            gallery
                .state
                .pattern_command_selection
                .select(cx, PatternCommand::OpenReview);
            gallery
                .state
                .pattern_branch_selection
                .select(cx, PatternBranch::GalleryPatterns);
        });

        let selected = app.update_entity(&root, |gallery, cx| {
            (
                gallery
                    .state
                    .pattern_command_selection
                    .get(cx)
                    .map(PatternCommand::label)
                    .unwrap_or("None"),
                gallery
                    .state
                    .pattern_branch_selection
                    .get(cx)
                    .map(PatternBranch::label)
                    .unwrap_or("None"),
            )
        });
        assert_eq!(selected, ("Open review", "gallery/patterns"));
    }
}
