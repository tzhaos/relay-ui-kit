use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, WindowSignalExt};

use crate::{
    components::display::CountBadge,
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius, space},
    tone::Tone,
};

/// A disclosure row for collapsible groups.
///
/// Three construction modes:
/// - `new` — host-owned: caller passes `open: bool` and handles toggling.
/// - `bound` — relay-bound: caller passes `Binding<bool>`, two-way binding.
/// - `stateful` — component-internal hooks: the component owns its open state
///   via `window.use_signal`, eliminating the need for the caller to manage
///   state. This is the React `useState` / Solid `createSignal` equivalent
///   for `RenderOnce` components.
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    label: String,
    open: bool,
    detail: Option<String>,
    count: Option<usize>,
    disabled: bool,
    binding: Option<Binding<bool>>,
    on_toggle: Option<ClickHandler>,
}

impl Disclosure {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>, open: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            open,
            detail: None,
            count: None,
            disabled: false,
            binding: None,
            on_toggle: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        label: impl Into<String>,
        binding: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            open: false,
            detail: None,
            count: None,
            disabled: false,
            binding: Some(binding),
            on_toggle: None,
        }
    }

    /// Create a disclosure that owns its open state internally via
    /// `window.use_signal`. The state persists across renders as long as the
    /// component keeps rendering in the same position (keyed by `id`).
    ///
    /// This is the component-internal hooks pattern — no `Binding` or host
    /// callback needed. The caller can optionally pass `on_toggle` to observe
    /// state changes.
    pub fn stateful(
        id: impl Into<ElementId>,
        label: impl Into<String>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let id = id.into();
        let binding = window.use_binding(id.clone(), cx, || false);
        Self {
            id,
            label: label.into(),
            open: false,
            detail: None,
            count: None,
            disabled: false,
            binding: Some(binding),
            on_toggle: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let open = binding.as_ref().map_or(self.open, |b| b.get(cx));
        let handler = self.on_toggle;
        let disabled = self.disabled;
        let interactive = !disabled && (binding.is_some() || handler.is_some());

        div()
            .id(self.id)
            .h(px(30.0))
            .px(px(space::SM))
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .text_color(theme.text_secondary)
            .tab_index(0)
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
            .when(interactive, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .child(
                Icon::new(if open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(IconSize::XSmall)
                .color(theme.text_muted),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label),
            )
            .when_some(self.detail, |this, detail| {
                this.child(
                    div()
                        .max_w(px(160.0))
                        .truncate()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(detail),
                )
            })
            .when_some(self.count, |this, count| {
                this.child(CountBadge::new(count).tone(Tone::Secondary))
            })
            .when(interactive, |this| {
                let binding_for_click = binding.clone();
                let binding_for_key = binding;
                let handler_for_click = handler.map(std::rc::Rc::new);
                let handler_for_key = handler_for_click.clone();
                this.on_click(move |event, window, cx| {
                    if let Some(binding) = &binding_for_click {
                        binding.update(cx, |open| {
                            *open = !*open;
                            true
                        });
                    }
                    if let Some(handler) = &handler_for_click {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key.as_str() == " "
                        || event.keystroke.key.as_str() == "enter"
                    {
                        if let Some(binding) = &binding_for_key {
                            binding.update(cx, |open| {
                                *open = !*open;
                                true
                            });
                        }
                        if let Some(handler) = &handler_for_key {
                            handler(&ClickEvent::default(), window, cx);
                        }
                        cx.stop_propagation();
                    }
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn disclosure_stores_open_state_from_host() {
        let disclosure = Disclosure::new("group", "Sessions", true);

        assert!(disclosure.open);
    }

    #[test]
    fn bound_disclosure_stores_binding() {
        let mut app = TestApp::new();
        let disclosure = app.update(|cx| Disclosure::bound("group", "Sessions", cx.binding(false)));

        assert!(disclosure.binding.is_some());
    }

    #[test]
    fn stateful_disclosure_uses_window_signal() {
        use gpui::{IntoElement, ParentElement, Render};
        use std::sync::{Arc, Mutex};

        // We render a Disclosure::stateful inside a view's render method.
        // The use_signal call happens during layout (inside render), which is
        // the only place use_keyed_state is valid.
        let binding_created = Arc::new(Mutex::new(false));

        struct HostView {
            binding_created: Arc<Mutex<bool>>,
        }
        impl HostView {
            fn new(cx: &mut gpui::Context<Self>, binding_created: Arc<Mutex<bool>>) -> Self {
                relay::init(cx);
                crate::styles::theme::init(cx);
                Self { binding_created }
            }
        }
        impl Render for HostView {
            fn render(
                &mut self,
                window: &mut Window,
                cx: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                // Create a stateful disclosure during render — use_signal
                // is called here, inside the layout phase. Context<Self>
                // derefs to &mut App.
                let disclosure = Disclosure::stateful("test-group", "Test", window, cx);
                *self.binding_created.lock().unwrap() = disclosure.binding.is_some();
                div().child(disclosure)
            }
        }

        let mut app = TestApp::new();
        let binding_created_clone = binding_created.clone();
        let mut window = app.open_window(|_, cx| HostView::new(cx, binding_created_clone));
        window.draw();

        assert!(
            *binding_created.lock().unwrap(),
            "stateful disclosure should have an internal binding after render"
        );
    }
}
