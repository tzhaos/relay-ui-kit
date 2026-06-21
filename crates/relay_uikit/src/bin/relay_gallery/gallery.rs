//! App-shaped gallery scenes for Relay UI components.
//!
//! Each scene presents components in the kind of surface where Relay will use
//! them, instead of packing every primitive into one long showcase page.

use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use relay::{
    Binding, Memo, ReactiveAppExt, ReactiveContextExt, Signal, SignalVecExt,
};
use relay_uikit::patterns::{ScrollSurface, SplitPaneState};
use relay_uikit::{ActiveTheme, TextInputState, TreeNode, space};

mod command_scene;
mod core_scene;
mod patterns_scene;
mod review_scene;
mod settings_scene;
mod shared;
mod stress_scene;
mod terminal_scene;
mod viewer_samples;
mod workbench_samples;

const FEEDBACK_TOAST_DURATION: Duration = Duration::from_secs(4);
const FEEDBACK_TOAST_LIMIT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySurface {
    Core,
    Patterns,
    Workbench,
    Stress,
}

pub struct GalleryScenesApp {
    pub surface: GallerySurface,
    pub state: GalleryState,
}

impl GalleryScenesApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            surface: GallerySurface::Core,
            state: GalleryState::new(cx),
        }
    }

    pub fn set_surface(&mut self, surface: GallerySurface) {
        self.surface = surface;
    }
}

/// Interactive state shared by the gallery scenes.
pub struct GalleryState {
    pub name_input: TextInputState,
    pub name_focus: FocusHandle,
    pub search_input: TextInputState,
    pub search_focus: FocusHandle,
    pub composer_input: TextInputState,
    pub composer_focus: FocusHandle,
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
    pub terminal_session: Binding<&'static str>,
    pub launcher_choice: Binding<&'static str>,
    pub branch_choice: Binding<&'static str>,
    pub branch_event: Signal<String>,
    pub viewer_tab: Binding<&'static str>,
    pub shell_split: Entity<SplitPaneState>,
    pub settings_select_open: Binding<bool>,
    pub ui_font_size: Binding<i32>,
    pub contrast: Binding<f32>,
    pub branch_picker_open: Binding<bool>,
    pub branch_actions_open: Binding<bool>,
    pub command_popover_open: Binding<bool>,
    pub command_context_open: Binding<bool>,
    pub confirm_dialog_open: Binding<bool>,

    pub pattern_dialog_open: Binding<bool>,
    /// Reactive toast list — mutations via `SignalVecExt` notify the view
    /// automatically, replacing the previous `Vec<FeedbackToast>` + manual
    /// `cx.notify()` pattern.
    pub feedback_toasts: Signal<Vec<FeedbackToast>>,
    pub feedback_toast_serial: u64,
    pub accent_choice: Signal<&'static str>,
    pub overlay_event: Signal<String>,
    pub core_disclosure_open: Binding<bool>,
    pub core_tree_src_open: Binding<bool>,
    pub core_tree_components_open: Binding<bool>,
    pub core_tree_list_open: Binding<bool>,
    /// Derived tree node list — recomputed only when the expand/selection
    /// signals change, instead of rebuilding on every render.
    pub core_tree_nodes: Memo<Vec<TreeNode>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedbackToast {
    pub id: u64,
}

impl FeedbackToast {
    fn new(id: u64) -> Self {
        Self { id }
    }
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryScenesApp>) -> Self {
        // Create the tree-expand bindings first so the derived tree node list
        // can subscribe to them.
        let core_tree_src_open: Binding<bool> = cx.binding(true);
        let core_tree_components_open: Binding<bool> = cx.binding(true);
        let core_tree_list_open: Binding<bool> = cx.binding(true);
        let viewer_tab_for_tree: Binding<&'static str> = cx.binding("diff");
        let core_tree_nodes = {
            let src = core_tree_src_open.clone();
            let components = core_tree_components_open.clone();
            let list = core_tree_list_open.clone();
            let viewer_tab = viewer_tab_for_tree.clone();
            cx.derived(move |cx| {
                build_core_tree_nodes(
                    src.get(cx),
                    components.get(cx),
                    list.get(cx),
                    viewer_tab.get(cx),
                )
            })
        };

        let state = Self {
            name_input: TextInputState::with_text("relay-agent"),
            name_focus: cx.focus_handle(),
            search_input: TextInputState::new(),
            search_focus: cx.focus_handle(),
            composer_input: TextInputState::new(),
            composer_focus: cx.focus_handle(),
            ui_font_size_input: cx.binding(TextInputState::with_text("14")),
            ui_font_size_focus: cx.focus_handle(),
            notifications: cx.binding(true),
            auto_archive: cx.binding(false),
            theme_choice: cx.binding("system"),
            filter_choice: cx.binding("all"),
            project_filter_choice: cx.binding("all-projects"),
            filter_menu_open: cx.binding(""),
            radio_choice: cx.binding("system"),
            seg_tab: cx.binding("diff"),
            terminal_session: cx.binding("codex"),
            launcher_choice: cx.binding("powershell"),
            branch_choice: cx.binding("ui-kit-branch-controls"),
            branch_event: cx.signal("Ready".into()),
            viewer_tab: viewer_tab_for_tree,
            shell_split: cx.new(|_| SplitPaneState::new(260.0)),
            settings_select_open: cx.binding(false),
            ui_font_size: cx.binding(14),
            contrast: cx.binding(60.0),
            branch_picker_open: cx.binding(false),
            branch_actions_open: cx.binding(false),
            command_popover_open: cx.binding(false),
            command_context_open: cx.binding(false),
            confirm_dialog_open: cx.binding(false),
            pattern_dialog_open: cx.binding(false),
            feedback_toasts: cx.signal(Vec::new()),
            feedback_toast_serial: 0,
            accent_choice: cx.signal("green"),
            overlay_event: cx.signal("No overlay action yet".into()),
            core_disclosure_open: cx.binding(true),
            core_tree_src_open: core_tree_src_open.clone(),
            core_tree_components_open: core_tree_components_open.clone(),
            core_tree_list_open: core_tree_list_open.clone(),
            core_tree_nodes,
        };

        // Declarative side effect: when the branch choice changes, derive the
        // branch event string automatically. This replaces the previous
        // pattern of setting `branch_event` imperatively inside every
        // `on_select` callback — now those callbacks only set `branch_choice`.
        {
            let branch_choice = state.branch_choice.clone();
            let branch_event = state.branch_event.clone();
            let _ = cx.watch(
                move |cx| {
                    let _ = branch_choice.get(cx);
                },
                move |cx| {
                    let key = branch_choice.get(cx);
                    branch_event.set_silent(cx, format!("Switched to {key}"));
                },
            );
        }

        state
    }
}

/// Build the core sample tree from the current expand/selection state.
///
/// Extracted as a pure function so the [`Memo`] compute closure stays small
/// and testable.
fn build_core_tree_nodes(
    src_open: bool,
    components_open: bool,
    list_open: bool,
    viewer_tab: &'static str,
) -> Vec<TreeNode> {
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
                .selected(viewer_tab == "tree:item"),
        );
    }

    nodes
}

impl GalleryScenesApp {
    pub fn add_feedback_toast(&mut self, cx: &mut Context<Self>) {
        self.state.feedback_toast_serial = self.state.feedback_toast_serial.wrapping_add(1);
        let id = self.state.feedback_toast_serial;
        // SignalVecExt::push notifies the view automatically — no cx.notify() needed.
        self.state.feedback_toasts.push(cx, FeedbackToast::new(id));
        // Enforce the limit by removing the oldest toast if exceeded.
        let len = self.state.feedback_toasts.read(cx, |toasts| toasts.len());
        if len > FEEDBACK_TOAST_LIMIT {
            self.state.feedback_toasts.remove(cx, 0);
        }
        self.schedule_feedback_toast_dismiss(id, cx);
    }

    pub fn dismiss_feedback_toast(&mut self, id: u64, cx: &mut App) {
        // SignalVecExt::retain notifies the view automatically.
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
                    // No cx.notify() — the retain() inside dismiss already
                    // notifies the view via the signal.
                    this.dismiss_feedback_toast(id, cx);
                });
            }
        })
        .detach();
    }
}

impl Render for GalleryScenesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface;
        let state = &self.state;
        let host = cx.entity();
        cx.tracked(|cx| render_surface(surface, state, &host, window, cx))
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
            .child(core_scene::render(state, host, theme, cx))
            .child(settings_scene::render(state, host, window, theme, cx))
            .into_any_element(),
        GallerySurface::Patterns => div()
            .flex()
            .flex_col()
            .gap(px(space::XL))
            .child(patterns_scene::render(state, host, theme, cx))
            .child(command_scene::render(state, host, theme, cx))
            .into_any_element(),
        GallerySurface::Workbench => div()
            .flex()
            .flex_col()
            .gap(px(space::XL))
            .child(terminal_scene::render(state, host, window, theme, cx))
            .child(review_scene::render(state, host, window, theme, cx))
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
