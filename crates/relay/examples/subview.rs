//! SubView — split a GPUI view into cached child entities.
//!
//! Run with `cargo run -p relay --example subview`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveAppExt, ReactiveView, Signal, SubView, init, view::reactive_render};

struct ShellView {
    header: SubView<HeaderPanel>,
    counter: SubView<CounterPanel>,
}

impl ShellView {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            header: SubView::new(cx, HeaderPanel::new),
            counter: SubView::new(cx, CounterPanel::new),
        }
    }
}

impl ReactiveView for ShellView {
    fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .size_full()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .child(
                self.header
                    .cached(gpui::StyleRefinement::default().w_full()),
            )
            .child(
                self.counter
                    .cached(gpui::StyleRefinement::default().w_full()),
            )
            .into_any_element()
    }
}

impl Render for ShellView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct HeaderPanel;

impl HeaderPanel {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl ReactiveView for HeaderPanel {
    fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(div().text_lg().child("Relay SubView"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child("Header and counter are separate cached GPUI entities."),
            )
            .into_any_element()
    }
}

impl Render for HeaderPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct CounterPanel {
    count: Signal<i32>,
}

impl CounterPanel {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            count: cx.signal(0),
        }
    }

    fn increment(&self, cx: &mut App) {
        self.count.update(cx, |count| {
            *count += 1;
            true
        });
    }
}

impl ReactiveView for CounterPanel {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .px_3()
            .py_2()
            .bg(rgb(0x27272a))
            .rounded(px(6.0))
            .child(div().child(format!("count: {}", self.count.get(cx))))
            .child(
                div()
                    .id("increment")
                    .px_3()
                    .py_1()
                    .bg(rgb(0x3b82f6))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x2563eb)))
                    .child("+1")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.increment(cx);
                    })),
            )
            .into_any_element()
    }
}

impl Render for CounterPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(420.0), px(220.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(ShellView::new),
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
