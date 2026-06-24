//! Relay UI gallery.
//!
//! A standalone, fully-interactive showcase app that proves the Relay UI crates
//! render and behave at Orca quality in GPUI. The gallery is a studio that
//! launches several small app-shaped scenes so components appear in the kind of
//! surface where Relay will use them:
//!
//! - **Core Kit** — primitives, inputs, forms, choices, feedback, lists, and rows.
//! - **Patterns Kit** — command, navigation, layout, overlay, and scroll patterns.
//! - **Stress Lab** — long labels, dense rows, disabled states, and overflow.
//! - **Workbench** — an app-like surface that composes Relay state primitives.
//!
//! Run with `cargo run -p relay_uikit --bin relay_gallery`.

mod gallery;
mod workbench_demo;

use gpui::{
    AnyElement, AnyView, App, AppContext, Bounds, Context, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowDecorations, WindowOptions, div, px, size,
};
use gpui_platform::application;
use relay::{
    ReactiveAppExt, Signal,
    view::{ReactiveView, reactive_render},
};
use relay_uikit::patterns::{TitleBar, WorkspaceBreadcrumb};
use relay_uikit::{ActiveTheme, Button, IconName, KitAssets, NavRow, space, theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Core,
    Patterns,
    Stress,
    Workbench,
}

pub struct GalleryApp {
    page: Signal<Page>,
    dark_mode: Signal<bool>,
    gallery: gpui::Entity<gallery::GalleryScenesApp>,
    workbench: gpui::Entity<workbench_demo::WorkbenchApp>,
}

impl GalleryApp {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: cx.signal(Page::Core),
            dark_mode: cx.signal(false),
            gallery: cx.new(gallery::GalleryScenesApp::new),
            workbench: cx.new(workbench_demo::WorkbenchApp::new),
        }
    }

    fn set_page(&self, page: Page, cx: &mut App) {
        self.page.set(cx, page);
        if let Some(surface) = page.gallery_surface() {
            self.gallery.update(cx, |gallery, cx| {
                gallery.set_surface(surface, cx);
            });
        }
    }

    fn top_bar(&self, cx: &mut App) -> impl IntoElement {
        let page = self.page.get(cx);
        TitleBar::new("Relay")
            .project("UI Kit")
            .center(WorkspaceBreadcrumb::new(vec![
                "Relay".into(),
                "Studio".into(),
                self.label_for(page).into(),
            ]))
    }

    fn label_for(&self, page: Page) -> &'static str {
        match page {
            Page::Core => "Core Kit",
            Page::Patterns => "Patterns Kit",
            Page::Stress => "Stress Lab",
            Page::Workbench => "Workbench",
        }
    }

    fn page_icon(page: Page) -> IconName {
        match page {
            Page::Core => IconName::LayoutGrid,
            Page::Patterns => IconName::Zap,
            Page::Stress => IconName::ListChecks,
            Page::Workbench => IconName::Terminal,
        }
    }

    fn coverage_count(page: Page) -> usize {
        match page {
            Page::Core => gallery::coverage_count(gallery::GallerySurface::Core),
            Page::Patterns => gallery::coverage_count(gallery::GallerySurface::Patterns),
            Page::Stress => gallery::coverage_count(gallery::GallerySurface::Stress),
            Page::Workbench => workbench_demo::COVERAGE_TITLES.len(),
        }
    }

    fn catalog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let page = self.page.get(cx);
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
            .child(self.catalog_row(Page::Core, page, cx))
            .child(self.catalog_row(Page::Patterns, page, cx))
            .child(self.catalog_row(Page::Stress, page, cx))
            .child(self.catalog_row(Page::Workbench, page, cx))
            .child(div().flex_1())
            .child(self.theme_toggle(cx))
    }

    fn theme_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let is_dark = self.dark_mode.get(cx);
        let label = if is_dark {
            "\u{263E}  Dark"
        } else {
            "\u{2600}  Light"
        };
        let dark_mode = self.dark_mode.clone();

        div()
            .px_3()
            .py_2()
            .border_t_1()
            .border_color(theme.border)
            .child(Button::new("theme-toggle", label).ghost().on_click(
                move |_event, _window, cx: &mut App| {
                    let was_dark = dark_mode.get(cx);
                    dark_mode.set(cx, !was_dark);
                    if was_dark {
                        theme::init(cx);
                    } else {
                        theme::init_dark(cx);
                    }
                },
            ))
    }

    fn catalog_row(&self, page: Page, current: Page, cx: &mut Context<Self>) -> impl IntoElement {
        NavRow::new(
            Self::page_key(page),
            Self::page_icon(page),
            self.label_for(page),
        )
        .selected(current == page)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_page(page, cx);
        }))
        .count(Self::coverage_count(page))
    }

    fn page_key(page: Page) -> &'static str {
        match page {
            Page::Core => "studio-core",
            Page::Patterns => "studio-patterns",
            Page::Stress => "studio-stress",
            Page::Workbench => "studio-workbench",
        }
    }
}

impl ReactiveView for GalleryApp {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let page = self.page.get(cx);
        let body = match page {
            Page::Workbench => cached_scene(self.workbench.clone()),
            Page::Core | Page::Patterns | Page::Stress => cached_scene(self.gallery.clone()),
        };

        div()
            .size_full()
            .bg(theme.app_bg)
            .text_color(theme.text)
            .font_family(theme::ui_family())
            .flex()
            .flex_col()
            .child(self.top_bar(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.catalog(cx))
                    .child(div().flex_1().min_w_0().min_h_0().child(body)),
            )
            .into_any_element()
    }
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

impl Page {
    fn gallery_surface(self) -> Option<gallery::GallerySurface> {
        match self {
            Page::Core => Some(gallery::GallerySurface::Core),
            Page::Patterns => Some(gallery::GallerySurface::Patterns),
            Page::Stress => Some(gallery::GallerySurface::Stress),
            Page::Workbench => None,
        }
    }
}

fn cached_scene(scene: impl Into<AnyView>) -> AnyElement {
    scene
        .into()
        .cached(gpui::StyleRefinement::default().size_full())
        .into_any_element()
}

fn main() {
    application().with_assets(KitAssets).run(|cx: &mut App| {
        theme::init(cx);
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        let Ok(_window) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(1180.0), px(780.0))),
                app_id: Some("relay-gallery".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(GalleryApp::new),
        ) else {
            eprintln!("relay_gallery: failed to open gallery window");
            return;
        };
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_counts_follow_surface_metadata() {
        assert_eq!(
            GalleryApp::coverage_count(Page::Core),
            gallery::coverage_count(gallery::GallerySurface::Core)
        );
        assert_eq!(
            GalleryApp::coverage_count(Page::Patterns),
            gallery::coverage_count(gallery::GallerySurface::Patterns)
        );
        assert_eq!(
            GalleryApp::coverage_count(Page::Stress),
            gallery::coverage_count(gallery::GallerySurface::Stress)
        );
        assert_eq!(
            GalleryApp::coverage_count(Page::Workbench),
            workbench_demo::COVERAGE_TITLES.len()
        );
    }
}
