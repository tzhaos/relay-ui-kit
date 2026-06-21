use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_uikit::patterns::{
    PaneToolbar, TopToolbar, WorkspaceBreadcrumb,
    display::KeyValue,
    layout::ListSection,
    navigation::{Tab, Tabs},
    overlay::{ContextMenu, Dialog, DropdownMenu, MenuItem, Select, SelectOption, TooltipBody},
    InputComposer, OutputLine, OutputLineStyle, OutputLog, OutputSurface, QuickAction, SessionRow,
    SourceView, TabStrip, TabToolbar, TaskRow, TaskRowData,
};
use relay_uikit::{Button, IconButton, IconName, Label, ListItem, TextInputState, Theme, Tone};

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
    let overlay_event_text = state.overlay_event.get(cx);

    let mut stack = scene_stack()
        .child(section(cx, "Layout patterns", layout_patterns(state, theme)))
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
    let output_body = output_patterns(state, theme, cx);
    let qa_body = quick_action_sample(state);
    let picker_body = picker_sample(state);
    let viewer_body = viewer_patterns(theme);

    stack = stack
        .child(section(cx, "Task Row & Session Row", rows_body))
        .child(section(cx, "Tab Strip & Toolbar", tabs_body))
        .child(section(cx, "Input Composer", composer_body))
        .child(section(cx, "Output Surface & Log", output_body))
        .child(section(cx, "Quick Actions", qa_body))
        .child(section(cx, "Item Picker", picker_body))
        .child(section(cx, "Source Viewer", viewer_body));

    stack
}

// ── Composite pattern samples ─────────────────────────────────────────────

fn row_patterns(state: &GalleryState, cx: &mut Context<GalleryScenesApp>) -> impl IntoElement + use<> {
    let active = state.seg_tab.get(cx) == "files";
    div().flex().flex_col().gap_2()
        .child(
            TaskRow::new("pat-task", TaskRowData {
                title: "Implement relay patterns".into(),
                status_label: "ACTIVE".into(),
                status_tone: Tone::Accent,
                branch: Some("relay/patterns".into()),
                changed: 5,
                review: 2,
            }).selected(true)
        )
        .child(
            SessionRow::new("pat-session", "codex", "relay/patterns")
                .status(Tone::Accent)
                .active(active)
        )
}

fn tab_patterns(state: &GalleryState, cx: &mut Context<GalleryScenesApp>) -> impl IntoElement + use<> {
    let active = state.seg_tab.get(cx);
    div().flex().flex_col().gap_2()
        .child(
            div().flex().gap_1()
                .child(TabStrip::new("pat-tab1", "Terminal")
                    .active(active == "files")
                    .status(Tone::Accent))
                .child(TabStrip::new("pat-tab2", "Preview")
                    .active(active == "diff")
                    .status(Tone::Muted))
                .child(TabStrip::new("pat-tab3", "Review").active(false))
        )
}

fn composer_sample(_cx: &mut Context<GalleryScenesApp>) -> impl IntoElement + use<> {
    div().h(px(40.0)).border_1().rounded(px(8.0)).border_color(gpui::rgb(0x333333)).px_2().flex().items_center()
        .child(div().text_sm().text_color(gpui::rgb(0x666666)).child("Type a message..."))
}

fn output_patterns(state: &GalleryState, theme: Theme, cx: &mut Context<GalleryScenesApp>) -> impl IntoElement + use<> {
    let connected = state.core_disclosure_open.get(cx);
    div().flex().flex_col().gap_2()
        .child(
            OutputSurface::new("pat-output",
                OutputLog::new(vec![
                    OutputLine::new("$ cargo build --release").style(OutputLineStyle::Input),
                    OutputLine::new("   Compiling relay-uikit v0.1.0").style(OutputLineStyle::Muted),
                    OutputLine::new("   Finished release in 12.4s").style(OutputLineStyle::Success),
                ]).prompt("> "),
            ).connected(connected)
        )
        .child(div().text_xs().text_color(theme.text_muted).child(format!("Connected: {connected}")))
}

fn quick_action_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().gap_2().flex_wrap()
        .child(QuickAction::new("qa-codex", "Launch Codex", "codex").on_click({
            let log = log.clone();
            move |_, _, cx: &mut gpui::App| { log.set(cx, "QuickAction: Launch Codex".into()); }
        }))
        .child(QuickAction::new("qa-build", "Build", "cargo build").on_click({
            move |_, _, cx: &mut gpui::App| { log.set(cx, "QuickAction: Build".into()); }
        }))
}

fn picker_sample(state: &GalleryState) -> impl IntoElement {
    div().flex().gap_2()
        .child(Button::new("picker-btn", "Open Picker").on_click({
            let picker = state.command_context_open.clone();
            move |_, _, cx: &mut gpui::App| { picker.update(cx, |v| { *v = !*v; true }); }
        }))
}

fn viewer_patterns(theme: Theme) -> impl IntoElement {
    div().flex().flex_col().gap_2().child(
        div().h(px(120.0)).border_1().border_color(theme.border).rounded(px(8.0)).overflow_hidden().child(
            SourceView::new(VIEWER_SAMPLE).language("rust")
        )
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
    div().max_w(px(640.0)).child(
        Tabs::bound(
            "patterns-navigation-tabs",
            vec![
                Tab::new("files", "Files").icon(IconName::FileText),
                Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                Tab::new("review", "Review")
                    .icon(IconName::MessageSquareText)
                    .count(3),
            ],
            state.seg_tab.clone(),
        ),
    )
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
        .child(
            div().w(px(260.0)).child(
                Select::bound(
                    "patterns-overlay-select",
                    state.theme_choice.clone(),
                    vec![
                        SelectOption::new("system", "System").detail("Follow OS appearance"),
                        SelectOption::new("light", "Light"),
                        SelectOption::new("dark", "Dark"),
                    ],
                ),
            ),
        )
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
                                open.update(cx, |v| { *v = !*v; true });
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
                    move |_window, cx| { open.set(cx, false); }
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
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        overlay_event.set(cx, message.to_string());
    }
}

fn close_dialog(
    state: &GalleryState,
    message: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let dialog_open = state.pattern_dialog_open.clone();
    let overlay_event = state.overlay_event.clone();
    move |_event, _window, cx| {
        dialog_open.set(cx, false);
        overlay_event.set(cx, message.to_string());
    }
}
