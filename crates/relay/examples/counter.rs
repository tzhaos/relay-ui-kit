#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, IntoElement, ParentElement, Render, Styled, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{Memo, ReactiveAppExt, ReactiveContextExt, Signal, init};

struct Counter {
    count: Signal<i32>,
    doubled: Memo<i32>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let count = cx.signal(0);
        let doubled = cx.memo({
            let count = count.clone();
            move |cx| count.get(cx) * 2
        });

        Self { count, doubled }
    }

    fn increment(&self, cx: &mut App) {
        self.count.update(cx, |count| {
            *count += 1;
            true
        });
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
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
                        .text_xl()
                        .child(format!("count: {}", self.count.get(cx))),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("doubled: {}", self.doubled.get(cx))),
                )
                .child(
                    div()
                        .id("increment")
                        .px_3()
                        .py_2()
                        .bg(rgb(0x3b82f6))
                        .hover(|style| style.bg(rgb(0x2563eb)))
                        .cursor_pointer()
                        .child("+1")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.increment(cx);
                        })),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(260.0), px(160.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(Counter::new),
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
