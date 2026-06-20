//! App-shaped gallery scenes for Relay UI components.
//!
//! Each scene presents components in the kind of surface where Relay will use
//! them, instead of packing every primitive into one long showcase page.

use gpui::{
    AnyElement, AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use relay_ui_core::{ActiveTheme, TextInputState, space};
use relay_ui_patterns::{ScrollSurface, SplitPaneState};

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
    pub ui_font_size_input: TextInputState,
    pub ui_font_size_focus: FocusHandle,
    pub notifications: bool,
    pub auto_archive: bool,
    pub theme_choice: &'static str,
    pub filter_choice: &'static str,
    pub project_filter_choice: &'static str,
    pub filter_menu_open: &'static str,
    pub radio_choice: &'static str,
    pub seg_tab: &'static str,
    pub terminal_session: &'static str,
    pub launcher_choice: &'static str,
    pub branch_choice: &'static str,
    pub branch_event: String,
    pub viewer_tab: &'static str,
    pub shell_split: Entity<SplitPaneState>,
    pub settings_select_open: bool,
    pub ui_font_size: i32,
    pub contrast: f32,
    pub branch_picker_open: bool,
    pub branch_actions_open: bool,
    pub command_popover_open: bool,
    pub command_context_open: bool,
    pub confirm_dialog_open: bool,
    pub pattern_select_open: bool,
    pub pattern_dialog_open: bool,
    pub feedback_toast_open: bool,
    pub feedback_toast_serial: u64,
    pub accent_choice: &'static str,
    pub overlay_event: String,
    pub core_disclosure_open: bool,
    pub core_tree_src_open: bool,
    pub core_tree_components_open: bool,
    pub core_tree_list_open: bool,
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryScenesApp>) -> Self {
        Self {
            name_input: TextInputState::with_text("relay-agent"),
            name_focus: cx.focus_handle(),
            search_input: TextInputState::new(),
            search_focus: cx.focus_handle(),
            composer_input: TextInputState::new(),
            composer_focus: cx.focus_handle(),
            ui_font_size_input: TextInputState::with_text("14"),
            ui_font_size_focus: cx.focus_handle(),
            notifications: true,
            auto_archive: false,
            theme_choice: "system",
            filter_choice: "all",
            project_filter_choice: "all-projects",
            filter_menu_open: "",
            radio_choice: "system",
            seg_tab: "diff",
            terminal_session: "codex",
            launcher_choice: "powershell",
            branch_choice: "ui-kit-branch-controls",
            branch_event: "Ready".into(),
            viewer_tab: "diff",
            shell_split: cx.new(|_| SplitPaneState::new(260.0)),
            settings_select_open: false,
            ui_font_size: 14,
            contrast: 60.0,
            branch_picker_open: false,
            branch_actions_open: false,
            command_popover_open: false,
            command_context_open: false,
            confirm_dialog_open: false,
            pattern_select_open: false,
            pattern_dialog_open: false,
            feedback_toast_open: true,
            feedback_toast_serial: 0,
            accent_choice: "green",
            overlay_event: "No overlay action yet".into(),
            core_disclosure_open: true,
            core_tree_src_open: true,
            core_tree_components_open: true,
            core_tree_list_open: true,
        }
    }
}

impl Render for GalleryScenesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let host = cx.entity();
        render_surface(self.surface, &self.state, &host, window, cx)
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
