//! App-shaped gallery scenes for Relay UI kit components.
//!
//! Each scene presents components in the kind of surface where Relay will use
//! them, instead of packing every primitive into one long showcase page.

use gpui::{
    AnyElement, AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use relay_ui_kit::{ActiveTheme, ScrollSurface, SplitPaneState, TextInputState, space};

mod command_scene;
mod foundations_scene;
mod product_samples;
mod review_scene;
mod settings_scene;
mod shared;
mod stress_scene;
mod terminal_scene;
mod viewer_samples;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySurface {
    Terminal,
    Review,
    Command,
    Settings,
    Foundations,
    Stress,
}

pub struct GalleryScenesApp {
    pub surface: GallerySurface,
    pub state: GalleryState,
}

impl GalleryScenesApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            surface: GallerySurface::Terminal,
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
    pub notifications: bool,
    pub auto_archive: bool,
    pub theme_choice: &'static str,
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
    pub overlay_event: String,
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryScenesApp>) -> Self {
        Self {
            name_input: TextInputState::with_text("relay-agent"),
            name_focus: cx.focus_handle(),
            search_input: TextInputState::new(),
            search_focus: cx.focus_handle(),
            notifications: true,
            auto_archive: false,
            theme_choice: "system",
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
            overlay_event: "No overlay action yet".into(),
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
        GallerySurface::Terminal => {
            terminal_scene::render(state, host, theme, cx).into_any_element()
        }
        GallerySurface::Review => {
            review_scene::render(state, host, window, theme, cx).into_any_element()
        }
        GallerySurface::Command => command_scene::render(state, host, theme, cx).into_any_element(),
        GallerySurface::Settings => {
            settings_scene::render(state, host, window, theme, cx).into_any_element()
        }
        GallerySurface::Foundations => {
            foundations_scene::render(state, host, theme, cx).into_any_element()
        }
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
