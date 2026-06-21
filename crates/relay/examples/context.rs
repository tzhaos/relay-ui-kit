//! Context — reactive provide/inject for cross-layer state sharing.
//!
//! `provide_context` installs a value keyed by type `T` as a GPUI global.
//! `use_context` reads it with dependency tracking — the consuming view
//! refreshes automatically when the provided value changes. This eliminates
//! prop-drilling for shared state like theme, locale, or active workspace.
//!
//! Run with `cargo run -p relay --example context`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{
    ContextHandle, ReactiveContextExt, init, provide_context, use_context,
};

/// The shared theme value provided via context.
#[derive(Clone, PartialEq)]
enum AppTheme {
    Light,
    Dark,
}

struct ContextDemo {
    theme_handle: ContextHandle<AppTheme>,
}

impl ContextDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let theme_handle = provide_context(cx, AppTheme::Dark);
        Self { theme_handle }
    }

    fn toggle_theme(&self, cx: &mut App) {
        let current = use_context::<AppTheme>(cx).unwrap_or(AppTheme::Dark);
        let next = match current {
            AppTheme::Light => AppTheme::Dark,
            AppTheme::Dark => AppTheme::Light,
        };
        self.theme_handle.set(cx, next);
    }
}

impl Render for ContextDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            // Read the theme from context — subscribes to changes.
            let theme = use_context::<AppTheme>(cx).unwrap_or(AppTheme::Dark);
            let (bg, text, label) = match theme {
                AppTheme::Light => (rgb(0xf4f4f5), rgb(0x202124), "Light"),
                AppTheme::Dark => (rgb(0x202124), rgb(0xf4f4f5), "Dark"),
            };

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(bg)
                .text_color(text)
                .child(div().text_lg().child("Context demo"))
                .child(div().text_sm().child(format!("Current theme (from context): {label}")))
                .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(
                    "The theme is provided via provide_context and read via use_context. \
                     No prop drilling — any child view can read it."
                ))
                .child(
                    div()
                        .id("toggle")
                        .px_3().py_2().bg(rgb(0x3b82f6)).rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x2563eb)))
                        .text_color(rgb(0xf4f4f5))
                        .child("Toggle theme")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_theme(cx);
                        })),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(360.0), px(220.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(ContextDemo::new),
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
