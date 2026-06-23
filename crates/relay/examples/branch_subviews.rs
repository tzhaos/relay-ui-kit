//! BranchSubViews - retain persistent branch panels as GPUI child entities.
//!
//! GPUI element state is tied to consecutive rendering of an element path. When
//! a branch owns view state, keep that branch as a `SubView` field in the host
//! entity and render only the active branch through GPUI's view cache.
//!
//! Run with `cargo run -p relay --example branch_subviews`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveAppExt, ReactiveView, Signal, SubView, init, view::reactive_render};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BranchKey {
    Plan,
    Review,
}

impl BranchKey {
    fn label(self) -> &'static str {
        match self {
            BranchKey::Plan => "Plan",
            BranchKey::Review => "Review",
        }
    }
}

struct BranchHost {
    active: Signal<BranchKey>,
    plan: SubView<BranchPanel>,
    review: SubView<BranchPanel>,
}

impl BranchHost {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            active: cx.signal(BranchKey::Plan),
            plan: SubView::new(cx, |cx| {
                BranchPanel::new(
                    cx,
                    BranchKey::Plan,
                    "Execution plan",
                    "State is owned by the Plan branch entity.",
                )
            }),
            review: SubView::new(cx, |cx| {
                BranchPanel::new(
                    cx,
                    BranchKey::Review,
                    "Review report",
                    "State is owned by the Review branch entity.",
                )
            }),
        }
    }

    fn set_active(&self, cx: &mut App, key: BranchKey) {
        self.active.set(cx, key);
    }

    fn active_view(&self, cx: &App) -> &SubView<BranchPanel> {
        match self.active.get(cx) {
            BranchKey::Plan => &self.plan,
            BranchKey::Review => &self.review,
        }
    }
}

impl ReactiveView for BranchHost {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active.get(cx);
        let plan_active = active == BranchKey::Plan;
        let review_active = active == BranchKey::Review;

        div()
            .id("branch-host")
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xf8fafc))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().text_lg().child("Branch subviews"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Active: {}", active.label())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        branch_button("show-plan", "Plan", plan_active).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.set_active(cx, BranchKey::Plan);
                            },
                        )),
                    )
                    .child(
                        branch_button("show-review", "Review", review_active).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.set_active(cx, BranchKey::Review);
                            }),
                        ),
                    ),
            )
            .child(
                self.active_view(cx)
                    .cached(gpui::StyleRefinement::default().w_full()),
            )
            .into_any_element()
    }
}

impl Render for BranchHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct BranchPanel {
    key: BranchKey,
    title: &'static str,
    detail: &'static str,
    counter: Signal<u32>,
    renders: usize,
}

impl BranchPanel {
    fn new(
        cx: &mut Context<Self>,
        key: BranchKey,
        title: &'static str,
        detail: &'static str,
    ) -> Self {
        init(cx);
        Self {
            key,
            title,
            detail,
            counter: cx.signal(0),
            renders: 0,
        }
    }

    fn increment(&self, cx: &mut App) {
        self.counter.update(cx, |counter| {
            *counter += 1;
            true
        });
    }

    #[cfg(test)]
    fn counter_value(&self) -> u32 {
        self.counter.get_untracked()
    }

    #[cfg(test)]
    fn render_count(&self) -> usize {
        self.renders
    }
}

impl ReactiveView for BranchPanel {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        self.renders += 1;
        let count = self.counter.get(cx);

        div()
            .id(format!("branch-panel-{}", self.key.label()))
            .min_h(px(140.0))
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .rounded(px(6.0))
            .border_1()
            .border_color(rgb(0x3f3f46))
            .bg(rgb(0x27272a))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().font_weight(FontWeight::MEDIUM).child(self.title))
                            .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(self.detail)),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded(px(999.0))
                            .bg(rgb(0x18181b))
                            .text_xs()
                            .text_color(rgb(0xcbd5e1))
                            .child(format!("renders: {}", self.renders)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().child(format!("local count: {count}")))
                    .child(action_button("increment", "+1").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.increment(cx);
                        },
                    ))),
            )
            .into_any_element()
    }
}

impl Render for BranchPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn branch_button(id: &'static str, label: &'static str, active: bool) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded(px(4.0))
        .bg(if active { rgb(0x2563eb) } else { rgb(0x3f3f46) })
        .hover(|style| style.bg(rgb(0x3b82f6)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn action_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded(px(4.0))
        .bg(rgb(0x3b82f6))
        .hover(|style| style.bg(rgb(0x2563eb)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(520.0), px(300.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(BranchHost::new),
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
    use gpui::{Entity, TestApp};

    use super::*;

    fn plan_entity(host: &BranchHost) -> Entity<BranchPanel> {
        host.plan.clone_entity()
    }

    fn review_entity(host: &BranchHost) -> Entity<BranchPanel> {
        host.review.clone_entity()
    }

    #[test]
    fn inactive_branch_does_not_render_until_selected() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| BranchHost::new(cx));
        let root = window.root();

        window.draw();
        let (plan, review) =
            app.update_entity(&root, |host, _cx| (plan_entity(host), review_entity(host)));

        let plan_renders = app.update_entity(&plan, |panel, _cx| panel.render_count());
        let review_renders = app.update_entity(&review, |panel, _cx| panel.render_count());
        assert_eq!(plan_renders, 1);
        assert_eq!(review_renders, 0);
    }

    #[test]
    fn branch_entity_state_survives_switching_away() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| BranchHost::new(cx));
        let root = window.root();

        window.draw();
        let plan = app.update_entity(&root, |host, _cx| plan_entity(host));
        app.update_entity(&plan, |panel, cx| {
            panel.increment(cx);
        });

        app.update_entity(&root, |host, cx| {
            host.set_active(cx, BranchKey::Review);
        });
        window.draw();

        app.update_entity(&root, |host, cx| {
            host.set_active(cx, BranchKey::Plan);
        });
        window.draw();

        let count = app.update_entity(&plan, |panel, _cx| panel.counter_value());
        assert_eq!(count, 1);
    }

    #[test]
    fn inactive_branch_stops_rendering_while_other_branch_is_active() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| BranchHost::new(cx));
        let root = window.root();

        window.draw();
        let (plan, review) =
            app.update_entity(&root, |host, _cx| (plan_entity(host), review_entity(host)));
        let plan_renders = app.update_entity(&plan, |panel, _cx| panel.render_count());

        app.update_entity(&root, |host, cx| {
            host.set_active(cx, BranchKey::Review);
        });
        window.draw();

        let plan_after_switch = app.update_entity(&plan, |panel, _cx| panel.render_count());
        let review_after_switch = app.update_entity(&review, |panel, _cx| panel.render_count());
        assert_eq!(plan_after_switch, plan_renders);
        assert_eq!(review_after_switch, 1);
    }
}
