//! Binding — two-way reactive value for form controls.
//!
//! A `Binding<T>` wraps a `Signal<T>` and is the primary type passed to
//! relay_uikit form components (Toggle, Slider, Select, TextInput, etc.).
//! Run with `cargo run -p relay --example binding`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{Binding, ReactiveAppExt, ReactiveContextExt, init};

struct BindingDemo {
    enabled: Binding<bool>,
    volume: Binding<f32>,
}

impl BindingDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            enabled: cx.binding(false),
            volume: cx.binding(50.0),
        }
    }
}

impl Render for BindingDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let enabled = self.enabled.get(cx);
            let volume = self.volume.get(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(
                    div()
                        .id("toggle")
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .child(
                            div()
                                .w(px(32.0))
                                .h(px(18.0))
                                .rounded(px(9.0))
                                .bg(if enabled {
                                    rgb(0x3b82f6)
                                } else {
                                    rgb(0x52525b)
                                })
                                .child(
                                    div()
                                        .size(px(14.0))
                                        .ml(px(1.0))
                                        .rounded_full()
                                        .bg(rgb(0xf4f4f5)),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .child(if enabled { "Enabled" } else { "Disabled" }),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.enabled.update(cx, |v| {
                                *v = !*v;
                                true
                            });
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("Volume: {volume:.0}")),
                )
                .child(
                    div()
                        .id("vol-up")
                        .px_3()
                        .py_2()
                        .bg(rgb(0x27272a))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x3f3f46)))
                        .child("+ Volume")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.volume.update(cx, |v| {
                                *v = (*v + 10.0).min(100.0);
                                true
                            });
                        })),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(280.0), px(180.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(BindingDemo::new),
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
