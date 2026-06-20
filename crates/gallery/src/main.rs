//! Relay UI gallery.
//!
//! A standalone, fully-interactive showcase app that proves the Relay UI crates
#![allow(clippy::expect_used)]
//! render and behave at Orca quality in GPUI. The gallery is a studio that
//! launches several small app-shaped scenes so components appear in the kind of
//! surface where Relay will use them:
//!
//! - **Workbench** — the Orca three-column shell (left rail / center terminal /
//!   right context). Click tasks to activate them, switch Files/Diff/Review and
//!   Terminal/Preview, filter the file tree, open the row context menu.
//! - **Terminal Hub** — terminal tabs, agent quick launch, and session UI.
//! - **Review** — file tree, Markdown/code preview, and diff review surfaces.
//! - **Command Center** — command palette, launcher, shortcuts, and menus.
//! - **Settings** — forms, choices, dropdowns, and feedback states.
//! - **Foundations** — buttons, icons, badges, rows, tabs, and empty states.
//! - **Stress Lab** — long labels, dense rows, disabled states, and overflow.
//!
//! Interactivity pattern: components carry view-free callbacks
//! (`Fn(&ClickEvent, &mut Window, &mut App)`). App-shaped scenes are GPUI child
//! entities so high-frequency interactions redraw the active surface instead of
//! the entire gallery shell.
//!
//! Run with `cargo run -p relay_gallery`.

mod gallery;
mod workbench_demo;

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, AppContext, Bounds, Context, IntoElement, ParentElement, Render,
    StyleRefinement, Styled, Window, WindowBounds, WindowDecorations, WindowOptions, div, px, size,
};
use gpui_platform::application;
use relay_ui_core::{ActiveTheme, Button, IconName, KitAssets, NavRow, space, theme};
use relay_ui_patterns::{TitleBar, WorkspaceBreadcrumb};

/// Which gallery page is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Workbench,
    Terminal,
    Review,
    Command,
    Settings,
    Foundations,
    Stress,
}

pub struct GalleryApp {
    page: Page,
    dark_mode: Rc<Cell<bool>>,
    gallery: gpui::Entity<gallery::GalleryScenesApp>,
    workbench: gpui::Entity<workbench_demo::WorkbenchApp>,
}

impl GalleryApp {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: Page::Workbench,
            dark_mode: Rc::new(Cell::new(false)),
            gallery: cx.new(gallery::GalleryScenesApp::new),
            workbench: cx.new(workbench_demo::WorkbenchApp::new),
        }
    }

    fn set_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        if let Some(surface) = page.gallery_surface() {
            self.gallery.update(cx, |gallery, cx| {
                gallery.set_surface(surface);
                cx.notify();
            });
        }
        cx.notify();
    }

    fn top_bar(&self) -> impl IntoElement {
        TitleBar::new("Relay")
            .project("UI Kit")
            .center(WorkspaceBreadcrumb::new(vec![
                "Relay".into(),
                "Studio".into(),
                self.page_label().into(),
            ]))
    }

    fn page_label(&self) -> &'static str {
        match self.page {
            Page::Workbench => "Workbench",
            Page::Terminal => "Terminal Hub",
            Page::Review => "Review Desk",
            Page::Command => "Command Center",
            Page::Settings => "Settings",
            Page::Foundations => "Foundation Lab",
            Page::Stress => "Stress Lab",
        }
    }

    fn page_icon(page: Page) -> IconName {
        match page {
            Page::Workbench => IconName::PanelLeft,
            Page::Terminal => IconName::Terminal,
            Page::Review => IconName::FileDiff,
            Page::Command => IconName::Zap,
            Page::Settings => IconName::Settings,
            Page::Foundations => IconName::LayoutGrid,
            Page::Stress => IconName::ListChecks,
        }
    }

    fn page_count(page: Page) -> Option<usize> {
        match page {
            Page::Review => Some(3),
            Page::Command => Some(4),
            Page::Stress => Some(9),
            _ => None,
        }
    }

    fn catalog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .w(px(space::RAIL_WIDTH))
            .flex_shrink_0()
            .h_full()
            .px_3()
            .py_3()
            .flex()
            .flex_col()
            .gap_1()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(self.catalog_row(Page::Workbench, cx))
            .child(self.catalog_row(Page::Terminal, cx))
            .child(self.catalog_row(Page::Review, cx))
            .child(self.catalog_row(Page::Command, cx))
            .child(self.catalog_row(Page::Settings, cx))
            .child(self.catalog_row(Page::Foundations, cx))
            .child(self.catalog_row(Page::Stress, cx))
            .child(div().flex_1())
            .child(self.theme_toggle(cx))
    }

    fn theme_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let is_dark = self.dark_mode.get();
        let label = if is_dark { "☀  Light" } else { "☾  Dark" };
        let dark_mode = self.dark_mode.clone();

        div()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(theme.border)
            .child(Button::new("theme-toggle", label).ghost().on_click(
                move |_event, _window, cx: &mut App| {
                    let was_dark = dark_mode.get();
                    dark_mode.set(!was_dark);
                    if was_dark {
                        theme::init(cx);
                    } else {
                        theme::init_dark(cx);
                    }
                },
            ))
    }

    fn catalog_row(&self, page: Page, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = NavRow::new(
            Self::page_key(page),
            Self::page_icon(page),
            self.page_label_for(page),
        )
        .selected(self.page == page)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_page(page, cx);
        }));
        if let Some(count) = Self::page_count(page) {
            row = row.count(count);
        }
        row
    }

    fn page_key(page: Page) -> &'static str {
        match page {
            Page::Workbench => "studio-workbench",
            Page::Terminal => "studio-terminal",
            Page::Review => "studio-review",
            Page::Command => "studio-command",
            Page::Settings => "studio-settings",
            Page::Foundations => "studio-foundations",
            Page::Stress => "studio-stress",
        }
    }

    fn page_label_for(&self, page: Page) -> &'static str {
        match page {
            Page::Workbench => "Workbench",
            Page::Terminal => "Terminal Hub",
            Page::Review => "Review Desk",
            Page::Command => "Command Center",
            Page::Settings => "Settings",
            Page::Foundations => "Foundation Lab",
            Page::Stress => "Stress Lab",
        }
    }
}

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let body = match self.page {
            Page::Workbench => cached_scene(self.workbench.clone()),
            Page::Terminal
            | Page::Review
            | Page::Command
            | Page::Settings
            | Page::Foundations
            | Page::Stress => cached_scene(self.gallery.clone()),
        };

        div()
            .size_full()
            .bg(theme.app_bg)
            .text_color(theme.text)
            .font_family(theme::ui_family())
            .flex()
            .flex_col()
            .child(self.top_bar())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.catalog(cx))
                    .child(div().flex_1().min_w_0().min_h_0().child(body)),
            )
    }
}

impl Page {
    fn gallery_surface(self) -> Option<gallery::GallerySurface> {
        match self {
            Page::Workbench => None,
            Page::Terminal => Some(gallery::GallerySurface::Terminal),
            Page::Review => Some(gallery::GallerySurface::Review),
            Page::Command => Some(gallery::GallerySurface::Command),
            Page::Settings => Some(gallery::GallerySurface::Settings),
            Page::Foundations => Some(gallery::GallerySurface::Foundations),
            Page::Stress => Some(gallery::GallerySurface::Stress),
        }
    }
}

fn cached_scene(scene: impl Into<AnyView>) -> AnyElement {
    scene
        .into()
        .cached(StyleRefinement::default().size_full())
        .into_any_element()
}

fn main() {
    application().with_assets(KitAssets).run(|cx: &mut App| {
        theme::init(cx);
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(1180.0), px(780.0))),
                app_id: Some("relay-gallery".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(GalleryApp::new),
        )
        .expect("open gallery window");
        cx.activate(true);
    });
}
