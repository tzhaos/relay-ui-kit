//! Effect cleanup - source-dependent side-effect lifetimes.
//!
//! `effect_in_with_cleanup` registers cleanup work for each effect run. The
//! cleanup runs before the effect re-runs and when the owning GPUI entity is
//! released.
//!
//! Run with `cargo run -p relay --example effect_cleanup`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, ParentElement, Render,
    Stateful, StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    ReactiveAppExt, ReactiveView, Signal, effect_in_with_cleanup, init, view::reactive_render,
};

struct EffectCleanupDemo {
    selected_channel: Signal<&'static str>,
    active_subscription: Signal<Option<&'static str>>,
    events: Signal<Vec<String>>,
}

impl EffectCleanupDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let selected_channel = cx.signal("inbox");
        let active_subscription = cx.signal(None);
        let events = cx.signal(Vec::new());

        let selected_for_effect = selected_channel.clone();
        let active_for_effect = active_subscription.clone();
        let events_for_effect = events.clone();
        let _ = effect_in_with_cleanup(cx, move |cx, cleanup| {
            let channel = selected_for_effect.get(cx);
            active_for_effect.set(cx, Some(channel));
            events_for_effect.update(cx, |events| {
                events.push(format!("subscribe {channel}"));
                true
            });

            let active_for_cleanup = active_for_effect.clone();
            let events_for_cleanup = events_for_effect.clone();
            cleanup.on_cleanup(move |cx| {
                active_for_cleanup.set(cx, None);
                events_for_cleanup.update(cx, |events| {
                    events.push(format!("unsubscribe {channel}"));
                    true
                });
            });
        });

        Self {
            selected_channel,
            active_subscription,
            events,
        }
    }

    fn cycle_channel(&self, cx: &mut App) {
        let current = self.selected_channel.get_untracked();
        let next = match current {
            "inbox" => "reviews",
            "reviews" => "alerts",
            _ => "inbox",
        };
        self.selected_channel.set(cx, next);
    }
}

impl ReactiveView for EffectCleanupDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected_channel.get(cx);
        let active = self.active_subscription.get(cx).unwrap_or("none");
        let events = self.events.get(cx);

        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .child(div().text_lg().child("Effect cleanup demo"))
            .child(
                div()
                    .text_sm()
                    .child(format!("Selected channel: {selected}")),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x4ade80))
                    .child(format!("Active subscription: {active}")),
            )
            .child(
                action_button("cycle-channel", "Cycle channel").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.cycle_channel(cx);
                    },
                )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(events.iter().rev().take(6).map(|event| {
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("* {event}"))
                    })),
            )
            .into_any_element()
    }
}

impl Render for EffectCleanupDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn action_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded(px(6.0))
        .bg(rgb(0x3b82f6))
        .hover(|style| style.bg(rgb(0x2563eb)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(420.0), px(300.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(EffectCleanupDemo::new),
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

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use super::*;

    #[test]
    fn source_change_cleans_previous_subscription_before_new_one() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| EffectCleanupDemo::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |demo, cx| demo.events.get(cx));
        assert_eq!(initial, vec!["subscribe inbox".to_string()]);

        app.update_entity(&root, |demo, cx| {
            demo.selected_channel.set(cx, "reviews");
        });

        let events = app.update_entity(&root, |demo, cx| demo.events.get(cx));
        assert_eq!(
            events,
            vec![
                "subscribe inbox".to_string(),
                "unsubscribe inbox".to_string(),
                "subscribe reviews".to_string(),
            ]
        );

        let active = app.update_entity(&root, |demo, cx| demo.active_subscription.get(cx));
        assert_eq!(active, Some("reviews"));
    }
}
