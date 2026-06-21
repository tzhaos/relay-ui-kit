//! App-shaped gallery scenes for Relay UI components.
//!
//! Each scene presents components in the kind of surface where Relay will use
//! them, instead of packing every primitive into one long showcase page.

use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement,
    Render, Styled, Window, div, px,
};
use relay::{
    Binding, Memo, ReactiveAppExt, ReactiveContextExt, Resource, Selector, Signal, SignalVecExt,
    view::StateScope,
};
use relay_uikit::patterns::{OutputLine, OutputLineStyle, ScrollSurface};
use relay_uikit::{ActiveTheme, TextInputState, TreeNode, space};

mod core_scene;
mod patterns_scene;
mod settings_scene;
mod shared;
mod stress_scene;

pub(super) const FEEDBACK_TOAST_DURATION: Duration = Duration::from_secs(4);
const FEEDBACK_TOAST_LIMIT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySurface {
    Core,
    Patterns,
    Stress,
}

pub struct GalleryScenesApp {
    pub surface: GallerySurface,
    pub state: GalleryState,
    pub scope: StateScope,
}

impl GalleryScenesApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut scope = StateScope::new();
        Self {
            surface: GallerySurface::Core,
            state: GalleryState::new(cx, &mut scope),
            scope,
        }
    }

    pub fn set_surface(&mut self, surface: GallerySurface) {
        self.surface = surface;
    }
}

/// Interactive state shared by the gallery scenes.
pub struct GalleryState {
    pub name_input: Binding<TextInputState>,
    pub name_focus: FocusHandle,
    pub search_input: Binding<TextInputState>,
    pub search_focus: FocusHandle,
    pub ui_font_size_input: Binding<TextInputState>,
    pub ui_font_size_focus: FocusHandle,
    pub notifications: Binding<bool>,
    pub auto_archive: Binding<bool>,
    pub theme_choice: Binding<&'static str>,
    pub filter_choice: Binding<&'static str>,
    pub project_filter_choice: Binding<&'static str>,
    pub filter_menu_open: Binding<&'static str>,
    pub radio_choice: Binding<&'static str>,
    pub seg_tab: Binding<&'static str>,
    pub settings_select_open: Binding<bool>,
    pub ui_font_size: Binding<i32>,
    pub contrast: Binding<f32>,
    pub command_popover_open: Binding<bool>,
    pub command_context_open: Binding<bool>,
    pub confirm_dialog_open: Binding<bool>,
    pub pattern_dialog_open: Binding<bool>,
    pub pattern_row_selection: Selector<&'static str>,
    pub pattern_tab_selection: Selector<&'static str>,
    pub pattern_output: Resource<Vec<OutputLine>, String>,
    pattern_output_refresh_serial: u64,
    stress_session_list: Entity<stress_scene::StressSessionList>,
    pub feedback_toasts: Signal<Vec<FeedbackToast>>,
    pub feedback_toast_serial: u64,
    pub accent_choice: Signal<&'static str>,
    pub overlay_event: Signal<String>,
    pub core_disclosure_open: Binding<bool>,
    pub core_tree_src_open: Binding<bool>,
    pub core_tree_components_open: Binding<bool>,
    pub core_tree_list_open: Binding<bool>,
    pub core_tree_nodes: Memo<Vec<TreeNode>>,
    pub settings_dirty: Memo<bool>,
}

#[derive(Debug, Clone)]
pub struct FeedbackToast {
    pub id: u64,
    pub message: String,
}

impl FeedbackToast {
    fn new(id: u64, message: impl Into<String>) -> Self {
        Self { id, message: message.into() }
    }
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryScenesApp>, scope: &mut StateScope) -> Self {
        let core_tree_src_open: Binding<bool> = cx.binding(true);
        let core_tree_components_open: Binding<bool> = cx.binding(true);
        let core_tree_list_open: Binding<bool> = cx.binding(true);
        let core_tree_nodes = {
            let src = core_tree_src_open.clone();
            let components = core_tree_components_open.clone();
            let list = core_tree_list_open.clone();
            cx.derived(move |cx| {
                build_core_tree_nodes(
                    src.get(cx),
                    components.get(cx),
                    list.get(cx),
                )
            })
        };

        let notifications: Binding<bool> = cx.binding(true);
        let auto_archive: Binding<bool> = cx.binding(false);
        let ui_font_size: Binding<i32> = cx.binding(14);
        let theme_choice: Binding<&'static str> = cx.binding("system");
        let settings_dirty = scope
            .form()
            .field("notifications", notifications.clone(), cx)
            .field("auto_archive", auto_archive.clone(), cx)
            .field("ui_font_size", ui_font_size.clone(), cx)
            .field("theme_choice", theme_choice.clone(), cx)
            .build_is_dirty(cx);

        Self {
            name_input: cx.binding(TextInputState::with_text("relay-agent")),
            name_focus: cx.focus_handle(),
            search_input: cx.binding(TextInputState::new()),
            search_focus: cx.focus_handle(),
            ui_font_size_input: cx.binding(TextInputState::with_text("14")),
            ui_font_size_focus: cx.focus_handle(),
            notifications,
            auto_archive,
            theme_choice,
            filter_choice: cx.binding("all"),
            project_filter_choice: cx.binding("all-projects"),
            filter_menu_open: cx.binding(""),
            radio_choice: cx.binding("system"),
            seg_tab: cx.binding("diff"),
            settings_select_open: cx.binding(false),
            ui_font_size,
            contrast: cx.binding(60.0),
            command_popover_open: cx.binding(false),
            command_context_open: cx.binding(false),
            confirm_dialog_open: cx.binding(false),
            pattern_dialog_open: cx.binding(false),
            pattern_row_selection: cx.selector(Some("task")),
            pattern_tab_selection: cx.selector(Some("terminal")),
            pattern_output: cx.ready_resource(initial_pattern_output_lines()),
            pattern_output_refresh_serial: 0,
            stress_session_list: cx.new(stress_scene::StressSessionList::new),
            feedback_toasts: cx.signal(Vec::new()),
            feedback_toast_serial: 0,
            accent_choice: cx.signal("green"),
            overlay_event: cx.signal("No overlay action yet".into()),
            core_disclosure_open: cx.binding(true),
            core_tree_src_open: core_tree_src_open.clone(),
            core_tree_components_open: core_tree_components_open.clone(),
            core_tree_list_open: core_tree_list_open.clone(),
            core_tree_nodes,
            settings_dirty,
        }
    }
}

fn initial_pattern_output_lines() -> Vec<OutputLine> {
    vec![
        OutputLine::new("$ cargo build --release").style(OutputLineStyle::Input),
        OutputLine::new("   Compiling relay-uikit v0.1.0").style(OutputLineStyle::Muted),
        OutputLine::new("   Finished release in 12.4s").style(OutputLineStyle::Success),
    ]
}

fn refreshed_pattern_output_lines(revision: u64) -> Vec<OutputLine> {
    vec![
        OutputLine::new(format!(
            "$ cargo check -p relay_uikit --bin relay_gallery #{revision:02}"
        ))
        .style(OutputLineStyle::Input),
        OutputLine::new("   Checking relay v0.1.0").style(OutputLineStyle::Muted),
        OutputLine::new("   Checking relay-uikit v0.1.0").style(OutputLineStyle::Muted),
        OutputLine::new("   Finished dev in 0.8s").style(OutputLineStyle::Success),
    ]
}

fn build_core_tree_nodes(src_open: bool, components_open: bool, list_open: bool) -> Vec<TreeNode> {
    use relay_uikit::IconName;

    let mut nodes = vec![TreeNode::new("tree:src", IconName::Folder, "src").expanded(src_open)];

    if src_open {
        nodes.push(
            TreeNode::new("tree:components", IconName::Folder, "components")
                .depth(1)
                .expanded(components_open),
        );
    }

    if src_open && components_open {
        nodes.push(
            TreeNode::new("tree:list", IconName::Folder, "list")
                .depth(2)
                .expanded(list_open),
        );
    }

    if src_open && components_open && list_open {
        nodes.push(
            TreeNode::new("tree:item", IconName::FileText, "item.rs")
                .depth(3)
                .selected(true),
        );
    }

    nodes
}

impl GalleryScenesApp {
    pub fn reload_pattern_output(&mut self, cx: &mut App) {
        self.state.pattern_output_refresh_serial =
            self.state.pattern_output_refresh_serial.wrapping_add(1);
        let revision = self.state.pattern_output_refresh_serial;
        let output = self.state.pattern_output.clone();
        output.reload(cx, move |cx| async move {
            cx.background_executor()
                .timer(Duration::from_millis(850))
                .await;
            Ok(refreshed_pattern_output_lines(revision))
        });
    }

    pub fn add_feedback_toast(&mut self, cx: &mut Context<Self>, message: impl Into<String>) {
        self.state.feedback_toast_serial = self.state.feedback_toast_serial.wrapping_add(1);
        let id = self.state.feedback_toast_serial;
        self.state.feedback_toasts.push(cx, FeedbackToast::new(id, message));
        let len = self.state.feedback_toasts.read(cx, |toasts| toasts.len());
        if len > FEEDBACK_TOAST_LIMIT {
            self.state.feedback_toasts.remove(cx, 0);
        }
        self.schedule_feedback_toast_dismiss(id, cx);
    }

    pub fn dismiss_feedback_toast(&mut self, id: u64, cx: &mut App) {
        self.state
            .feedback_toasts
            .retain(cx, |toast| toast.id != id);
    }

    fn schedule_feedback_toast_dismiss(&self, id: u64, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(FEEDBACK_TOAST_DURATION)
                .await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.dismiss_feedback_toast(id, cx);
                });
            }
        })
        .detach();
    }
}

impl Render for GalleryScenesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let surface = self.surface;
            let state = &self.state;
            let host = cx.entity();
            render_surface(surface, state, &host, window, cx)
        })
    }
}

fn render_surface(
    surface: GallerySurface,
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    cx: &mut Context<GalleryScenesApp>,
) -> AnyElement {
    let theme = *cx.theme();
    let content: AnyElement = match surface {
        GallerySurface::Core => div()
            .flex()
            .flex_col()
            .gap(px(space::XL))
            .child(core_scene::render(state, host, window, theme, cx))
            .child(settings_scene::render(state, host, window, theme, cx))
            .into_any_element(),
        GallerySurface::Patterns => div()
            .flex()
            .flex_col()
            .gap(px(space::XL))
            .child(patterns_scene::render(state, host, theme, cx))
            .into_any_element(),
        GallerySurface::Stress => stress_scene::render(state, host, theme, cx).into_any_element(),
    };

    div()
        .size_full()
        .bg(theme.app_bg)
        .child(
            ScrollSurface::new(
                "gallery-scroll",
                div()
                    .max_w(px(1160.0))
                    .mx_auto()
                    .p(px(space::XL))
                    .child(content),
            )
            .reserve_gutter(true),
        )
        .into_any_element()
}
