//! Form — aggregate bound fields with derived dirty-checking, reset, and commit.
//!
//! `Form` collects multiple `Binding<T>` fields and derives `is_dirty` as a
//! `Memo<bool>`. It also provides `reset` (restore initial values) and
//! `commit` (snapshot current values as the new clean baseline).
//!
//! Run with `cargo run -p relay --example form`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{Binding, Form, Memo, ReactiveAppExt, ReactiveContextExt, init};

struct FormDemo {
    name: Binding<String>,
    enabled: Binding<bool>,
    count: Binding<i32>,
    is_dirty: Memo<bool>,
    _form: std::mem::ManuallyDrop<Form>,
}

impl FormDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let name: Binding<String> = cx.binding("Alice".into());
        let enabled: Binding<bool> = cx.binding(true);
        let count: Binding<i32> = cx.binding(42);

        let mut form = Form::new();
        form.field("name", name.clone(), cx);
        form.field("enabled", enabled.clone(), cx);
        form.field("count", count.clone(), cx);
        let is_dirty = form.build_is_dirty(cx);

        Self {
            name,
            enabled,
            count,
            is_dirty,
            _form: std::mem::ManuallyDrop::new(form),
        }
    }
}

impl Render for FormDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let name = self.name.get(cx);
            let enabled = self.enabled.get(cx);
            let count = self.count.get(cx);
            let dirty = self.is_dirty.get(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Form demo"))
                .child(
                    div()
                        .px_3().py_2().rounded(px(6.0))
                        .bg(if dirty { rgb(0xf59e0b) } else { rgb(0x27272a) })
                        .text_color(if dirty { rgb(0x202124) } else { rgb(0xa1a1aa) })
                        .text_sm()
                        .child(if dirty {
                            "Unsaved changes (derived via Form::is_dirty)"
                        } else {
                            "No changes (clean)"
                        }),
                )
                .child(div().text_sm().child(format!("Name: {name}")))
                .child(div().text_sm().child(format!("Enabled: {enabled}")))
                .child(div().text_sm().child(format!("Count: {count}")))
                .child(
                    div().flex().gap_2().flex_wrap()
                        .child(
                            div()
                                .id("inc-count")
                                .px_3().py_2().bg(rgb(0x3b82f6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Count + 1")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.count.update(cx, |v| { *v += 1; true });
                                })),
                        )
                        .child(
                            div()
                                .id("toggle-enabled")
                                .px_3().py_2().bg(rgb(0x8b5cf6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x7c3aed)))
                                .child("Toggle enabled")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.enabled.update(cx, |v| { *v = !*v; true });
                                })),
                        )
                        .child(
                            div()
                                .id("reset")
                                .px_3().py_2().bg(rgb(0xef4444)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xdc2626)))
                                .child("Reset (restore initial)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this._form.reset(cx);
                                })),
                        )
                        .child(
                            div()
                                .id("commit")
                                .px_3().py_2().bg(rgb(0x4ade80)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x22c55e)))
                                .text_color(rgb(0x202124))
                                .child("Commit (save baseline)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this._form.commit(cx);
                                })),
                        ),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(380.0), px(300.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(FormDemo::new),
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
