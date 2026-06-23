use gpui::{
    AnyElement, AnyView, App, ClickEvent, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use relay::{
    KeyedSubViews, OrderedSelectionModel, ReactiveAppExt, ReactiveView, SelectionReconcilePolicy,
    Selector, Signal, SignalVecExt, use_ordered_selection_model, view::reactive_render,
};
use relay_uikit::patterns::{
    CommandPalette, CommandRow, ItemPicker, KeybindingShortcut, OutputLog, OutputSurface,
    PaneToolbar, PickerAction, PickerOption, QuickAction, SessionRow, SourceView, TabStrip,
    TaskRow, TaskRowData, TopToolbar, WorkspaceBreadcrumb,
    display::KeyValue,
    layout::ListSection,
    navigation::{Tab, Tabs},
    output_resource_snapshot,
    overlay::{ContextMenu, Dialog, DropdownMenu, MenuItem, Select, SelectOption, TooltipBody},
};
use relay_uikit::{
    ActiveTheme, Button, IconButton, IconName, Label, ListItem, ListItemSpacing, StatusDot, Theme,
    ThemePreviewKind, Tone,
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

    let mut stack = scene_stack()
        .child(section(
            cx,
            "Layout patterns",
            layout_patterns(state, theme),
        ))
        .child(section(
            cx,
            "Display patterns",
            display_patterns(state, theme),
        ))
        .child(section(
            cx,
            "Navigation patterns",
            navigation_patterns(state),
        ));
    let overlay_body = overlay_patterns(state, theme, &overlay_event_text, cx);
    stack = stack.child(section(cx, "Overlay patterns", overlay_body));

    if state.pattern_dialog_open.get(cx) {
        stack = stack.child(settings_dialog(state));
    }

    // Composite pattern demos — extract bodies first to avoid borrow conflicts
    let rows_body = row_patterns(state, cx);
    let tabs_body = tab_patterns(state, cx);
    let composer_body = composer_sample(cx);
    let output_body = output_patterns(state, host, theme, cx);
    let qa_body = quick_action_sample(state);
    let command_picker_body = command_picker_patterns(state, theme, cx);
    let picker_body = picker_sample(state);
    let viewer_body = viewer_patterns(theme);

    stack = stack
        .child(section(cx, "Task Row & Session Row", rows_body))
        .child(section(cx, "Tab Strip & Toolbar", tabs_body))
        .child(section(cx, "Input Composer", composer_body))
        .child(section(cx, "Output Surface & Log", output_body))
        .child(section(cx, "Quick Actions", qa_body))
        .child(section(cx, "Command Palette & Picker", command_picker_body))
        .child(section(cx, "Item Picker", picker_body))
        .child(section(cx, "Source Viewer", viewer_body));

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
        div()
            .flex()
            .gap_1()
            .child(
                TabStrip::new("pat-tab1", "Terminal")
                    .active_by(
                        state.pattern_tab_selection.clone(),
                        PatternPreviewTab::Terminal,
                    )
                    .status(Tone::Accent),
            )
            .child(
                TabStrip::new("pat-tab2", "Preview")
                    .active_by(
                        state.pattern_tab_selection.clone(),
                        PatternPreviewTab::Preview,
                    )
                    .status(Tone::Muted),
            )
            .child(TabStrip::new("pat-tab3", "Review").active_by(
                state.pattern_tab_selection.clone(),
                PatternPreviewTab::Review,
            )),
    )
}

fn composer_sample(_cx: &mut Context<GalleryScenesApp>) -> impl IntoElement + use<> {
    div()
        .h(px(40.0))
        .border_1()
        .rounded(px(8.0))
        .border_color(gpui::rgb(0x333333))
        .px_2()
        .flex()
        .items_center()
        .child(
            div()
                .text_sm()
                .text_color(gpui::rgb(0x666666))
                .child("Type a message..."),
        )
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

fn viewer_patterns(theme: Theme) -> impl IntoElement {
    div().flex().flex_col().gap_2().child(
        div()
            .h(px(120.0))
            .border_1()
            .border_color(theme.border)
            .rounded(px(8.0))
            .overflow_hidden()
            .child(SourceView::new(VIEWER_SAMPLE).language("rust")),
    )
}

const VIEWER_SAMPLE: &str = "pub struct OutputLine {\n    text: String,\n    style: OutputLineStyle,\n}\n\nimpl OutputLine {\n    pub fn new(text: impl Into<String>) -> Self {\n        Self { text: text.into(), style: OutputLineStyle::Output }\n    }\n}";

fn layout_patterns(state: &GalleryState, theme: Theme) -> impl IntoElement {
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
            strip()
                .child(
                    Button::new("layout-left-action", "Focus")
                        .icon(IconName::PanelLeft)
                        .on_click(pattern_event(state, "Layout toolbar focused")),
                )
                .child(
                    Button::new("layout-right-action", "Refresh")
                        .icon(IconName::RefreshCw)
                        .on_click(pattern_event(state, "Layout toolbar refreshed")),
                ),
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
        .min_h(px(188.0))
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

fn menu_action(state: &GalleryState, label: &'static str, icon: IconName) -> MenuItem {
    MenuItem::new(label)
        .icon(icon)
        .on_click(pattern_event(state, label))
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
