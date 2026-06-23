use gpui::{
    App, ClickEvent, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, KeyHandler, SharedChangeHandler},
    theme::{ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY, radius},
};

use super::{
    InputActionKind, InputValueKind, TextInputState,
    platform_input::{
        AfterEdit, PlatformInputMode, PointerSelectionState, SingleLineInputStyle,
        single_line_input,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberInputLayout {
    ControlsAroundValue,
    ControlsTrailing,
}

#[derive(Clone, Copy)]
enum NumberStep {
    Decrement,
    Increment,
}

struct EditableNumberValue {
    focus: FocusHandle,
    before: String,
    after: String,
    focused: bool,
    key_context: &'static str,
    binding: Option<Binding<TextInputState>>,
    on_key: Option<KeyHandler>,
}

/// A compact numeric input with optional stepper callbacks.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    value: i32,
    value_binding: Option<Binding<i32>>,
    suffix: Option<String>,
    layout: NumberInputLayout,
    editable: Option<EditableNumberValue>,
    disabled: bool,
    min: Option<i32>,
    max: Option<i32>,
    step: i32,
    on_change: Option<SharedChangeHandler<i32>>,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
}

impl NumberInput {
    pub fn new(id: impl Into<ElementId>, value: i32) -> Self {
        Self {
            id: id.into(),
            value,
            value_binding: None,
            suffix: None,
            layout: NumberInputLayout::ControlsAroundValue,
            editable: None,
            disabled: false,
            min: None,
            max: None,
            step: 1,
            on_change: None,
            on_decrement: None,
            on_increment: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, binding: Binding<i32>) -> Self {
        Self {
            id: id.into(),
            value: 0,
            value_binding: Some(binding),
            suffix: None,
            layout: NumberInputLayout::ControlsAroundValue,
            editable: None,
            disabled: false,
            min: None,
            max: None,
            step: 1,
            on_change: None,
            on_decrement: None,
            on_increment: None,
        }
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn layout(mut self, layout: NumberInputLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Disable the number input — blocks stepper buttons and keyboard input.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn range(mut self, min: i32, max: i32) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    pub fn min(mut self, min: i32) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: i32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: i32) -> Self {
        self.step = step.max(1);
        self
    }

    pub fn editable(mut self, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        self.editable = Some(EditableNumberValue {
            focus,
            before: before.to_string(),
            after: after.to_string(),
            focused: false,
            key_context: "NumberInput",
            binding: None,
            on_key: None,
        });
        self
    }

    pub fn input(self, focus: FocusHandle, state: &TextInputState) -> Self {
        self.editable(focus, state)
    }

    pub fn editable_bound(mut self, focus: FocusHandle, binding: Binding<TextInputState>) -> Self {
        self.editable = Some(EditableNumberValue {
            focus,
            before: String::new(),
            after: String::new(),
            focused: false,
            key_context: "NumberInput",
            binding: Some(binding),
            on_key: None,
        });
        self
    }

    pub fn input_bound(self, focus: FocusHandle, binding: Binding<TextInputState>) -> Self {
        self.editable_bound(focus, binding)
    }

    pub fn focused(mut self, focused: bool) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.focused = focused;
        }
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.key_context = key_context;
        }
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.on_key = Some(Box::new(handler));
        }
        self
    }

    pub fn on_change(mut self, handler: impl Fn(i32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_decrement(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_decrement = Some(Box::new(handler));
        self
    }

    pub fn on_increment(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_increment = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let NumberInput {
            id,
            value: current_value,
            value_binding,
            suffix,
            layout,
            editable,
            disabled,
            min,
            max,
            step,
            on_change,
            on_decrement,
            on_increment,
        } = self;
        let theme = *cx.theme();
        let pointer = editable.as_ref().and_then(|editable| {
            editable.binding.as_ref().map(|_| {
                window.use_keyed_state((id.clone(), "pointer-selection"), cx, |_, _| {
                    PointerSelectionState::default()
                })
            })
        });
        let current_value = value_binding
            .as_ref()
            .map_or(current_value, |binding| binding.get(cx));
        let editable_text_binding = editable
            .as_ref()
            .and_then(|editable| editable.binding.clone());
        let value_id = format!("{id}-value");
        let value = match editable {
            Some(mut editable) => {
                editable.focused = editable.focused || editable.focus.is_focused(window);
                if let Some(binding) = &editable.binding {
                    let state = binding.get(cx);
                    let (before, after) = state.split();
                    editable.before = before.to_string();
                    editable.after = after.to_string();
                }
                editable_number_value(
                    value_id,
                    editable,
                    pointer,
                    current_value,
                    suffix,
                    theme,
                    value_binding.clone(),
                    min,
                    max,
                    on_change.clone(),
                )
                .into_any_element()
            }
            None => {
                number_value(current_value, suffix, theme.text, theme.text_muted).into_any_element()
            }
        };
        let decrement = stepper(
            (id.clone(), "decrement"),
            IconName::Minus,
            on_decrement,
            value_binding.clone(),
            editable_text_binding.clone(),
            on_change.clone(),
            NumberStep::Decrement,
            current_value,
            min,
            max,
            step,
            theme.hover,
            theme.text_muted,
        );
        let increment = stepper(
            (id.clone(), "increment"),
            IconName::Plus,
            on_increment,
            value_binding,
            editable_text_binding,
            on_change,
            NumberStep::Increment,
            current_value,
            min,
            max,
            step,
            theme.hover,
            theme.text_muted,
        );

        div()
            .id(id)
            .h(px(30.0))
            .min_w(px(108.0))
            .flex()
            .items_center()
            .overflow_hidden()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
            .map(|this| match layout {
                NumberInputLayout::ControlsAroundValue => {
                    this.child(decrement).child(value).child(increment)
                }
                NumberInputLayout::ControlsTrailing => this.child(value).child(
                    div()
                        .h_full()
                        .border_l_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .child(decrement)
                        .child(
                            div()
                                .h(px(16.0))
                                .w(px(BORDER_WIDTH))
                                .bg(theme.border.opacity(0.7)),
                        )
                        .child(increment),
                ),
            })
    }
}

fn number_value(
    value: i32,
    suffix: Option<String>,
    text_color: gpui::Hsla,
    suffix_color: gpui::Hsla,
) -> impl IntoElement {
    div()
        .min_w(px(58.0))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_color)
        .child(value.to_string())
        .when_some(suffix, |this, suffix| {
            this.child(div().text_xs().text_color(suffix_color).child(suffix))
        })
}

#[expect(
    clippy::too_many_arguments,
    reason = "Editable number rendering keeps GPUI element state and relay bindings explicit."
)]
fn editable_number_value(
    id: impl Into<ElementId>,
    editable: EditableNumberValue,
    pointer: Option<gpui::Entity<PointerSelectionState>>,
    fallback: i32,
    suffix: Option<String>,
    theme: crate::Theme,
    value_binding: Option<Binding<i32>>,
    min: Option<i32>,
    max: Option<i32>,
    on_change: Option<SharedChangeHandler<i32>>,
) -> impl IntoElement {
    let id = id.into();
    let focus_for_click = editable.focus.clone();
    let focus_for_mouse_down = editable.focus.clone();
    let is_focused = editable.focused;
    let text_binding = editable.binding;
    let on_key = editable.on_key;
    let show_fallback = editable.before.is_empty() && editable.after.is_empty() && !is_focused;
    let allow_negative = min.is_none_or(|min| min < 0);
    let handle_key = text_binding.is_some() || on_key.is_some();
    let after_edit = text_binding.as_ref().map(|_| {
        let value_binding = value_binding.clone();
        let on_change = on_change.clone();
        std::rc::Rc::new(
            move |text_binding: &Binding<TextInputState>, window: &mut Window, cx: &mut App| {
                apply_parsed_integer_value(
                    text_binding,
                    &value_binding,
                    &on_change,
                    fallback,
                    min,
                    max,
                    false,
                    window,
                    cx,
                );
            },
        ) as AfterEdit
    });

    div()
        .id(id.clone())
        .min_w(px(58.0))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .border_l_1()
        .border_r_1()
        .border_color(theme.border)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .track_focus(&editable.focus)
        .tab_index(0)
        .key_context(editable.key_context)
        .cursor(gpui::CursorStyle::IBeam)
        .when(is_focused, |this| this.bg(theme.accent_bg))
        .when(show_fallback, |this| {
            this.text_color(theme.text_muted)
                .child(fallback.to_string())
        })
        .when(!show_fallback, |this| {
            if let (Some(text_binding), Some(pointer)) = (text_binding.clone(), pointer.clone()) {
                this.text_color(theme.text).child(single_line_input(
                    (id.clone(), "editor"),
                    editable.focus.clone(),
                    text_binding,
                    pointer,
                    SingleLineInputStyle {
                        text_color: theme.text,
                        placeholder_color: theme.text_muted,
                        selection_color: theme.selection,
                        cursor_color: theme.accent,
                    },
                    fallback.to_string(),
                    show_fallback,
                    false,
                    PlatformInputMode::Integer { allow_negative },
                    after_edit.clone(),
                ))
            } else {
                this.text_color(theme.text)
                    .child(editable.before)
                    .when(is_focused, |this| {
                        this.child(div().w(px(1.5)).h(px(16.0)).bg(theme.accent))
                    })
                    .child(editable.after)
            }
        })
        .when_some(suffix, |this, suffix| {
            this.child(div().text_xs().text_color(theme.text_muted).child(suffix))
        })
        .when(handle_key, |this| {
            this.on_key_down(move |event, window, cx| {
                let mut consumed = false;
                if let Some(binding) = &text_binding {
                    consumed = handle_bound_integer_platform_key(
                        binding,
                        &value_binding,
                        &on_change,
                        event,
                        fallback,
                        min,
                        max,
                        allow_negative,
                        window,
                        cx,
                    );
                }
                if let Some(on_key) = &on_key {
                    on_key(event, window, cx);
                }
                if consumed {
                    cx.stop_propagation();
                }
            })
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.focus(&focus_for_mouse_down, cx);
        })
        .on_mouse_down_out(|_, window, _cx| {
            window.blur();
        })
        .on_click(move |_, window, cx| {
            window.focus(&focus_for_click, cx);
        })
}

#[expect(
    clippy::too_many_arguments,
    reason = "GPUI event wiring passes a compact set of render-time handles."
)]
fn stepper(
    id: impl Into<ElementId>,
    icon: IconName,
    handler: Option<ClickHandler>,
    value_binding: Option<Binding<i32>>,
    text_binding: Option<Binding<TextInputState>>,
    on_change: Option<SharedChangeHandler<i32>>,
    direction: NumberStep,
    current_value: i32,
    min: Option<i32>,
    max: Option<i32>,
    step: i32,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    let interactive = handler.is_some() || value_binding.is_some();

    div()
        .id(id)
        .w(px(24.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .when(interactive, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
                .on_click(move |event, window, cx| {
                    let next = stepped_value(current_value, direction, step, min, max);
                    if next != current_value {
                        if let Some(binding) = &value_binding {
                            binding.set(cx, next);
                        }
                        if let Some(binding) = &text_binding {
                            sync_text_binding(binding, cx, next);
                        }
                        if let Some(handler) = &on_change {
                            handler(next, window, cx);
                        }
                    }
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
        })
        .child(Icon::new(icon).size(IconSize::Small).color(color))
}

#[expect(
    clippy::too_many_arguments,
    reason = "Keyboard handling bridges text, value, and optional host callbacks."
)]
fn handle_bound_integer_platform_key(
    text_binding: &Binding<TextInputState>,
    value_binding: &Option<Binding<i32>>,
    on_change: &Option<SharedChangeHandler<i32>>,
    event: &KeyDownEvent,
    current_value: i32,
    min: Option<i32>,
    max: Option<i32>,
    allow_negative: bool,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let _allow_negative = allow_negative;
    let mut consumed = false;
    let mut should_parse = false;
    let mut should_sync_text = false;
    text_binding.update(cx, |state| {
        let action = state.handle_platform_key(event);
        consumed = action.should_notify();
        match action.contract_kind_for(InputValueKind::Number) {
            InputActionKind::Changed(InputValueKind::Number) => {
                should_parse = true;
            }
            InputActionKind::Submit | InputActionKind::Validate => {
                should_parse = true;
                should_sync_text = true;
            }
            InputActionKind::Cancel => {
                state.set_text(current_value.to_string());
            }
            InputActionKind::CursorMoved
            | InputActionKind::Ignored
            | InputActionKind::Changed(_) => {}
        }
        action.should_notify()
    });

    if should_parse {
        apply_parsed_integer_value(
            text_binding,
            value_binding,
            on_change,
            current_value,
            min,
            max,
            should_sync_text,
            window,
            cx,
        );
    }

    consumed
}

#[expect(
    clippy::too_many_arguments,
    reason = "Numeric input sync depends on host value binding, range, and submit semantics."
)]
fn apply_parsed_integer_value(
    text_binding: &Binding<TextInputState>,
    value_binding: &Option<Binding<i32>>,
    on_change: &Option<SharedChangeHandler<i32>>,
    current_value: i32,
    min: Option<i32>,
    max: Option<i32>,
    should_sync_text: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(value) = parse_integer(text_binding.get(cx).value(), min, max) else {
        return;
    };

    if let Some(binding) = value_binding {
        binding.set(cx, value);
    }
    if should_sync_text {
        sync_text_binding(text_binding, cx, value);
    }
    if value != current_value
        && let Some(handler) = on_change
    {
        handler(value, window, cx);
    }
}

fn parse_integer(text: &str, min: Option<i32>, max: Option<i32>) -> Option<i32> {
    text.parse::<i32>()
        .ok()
        .map(|value| clamp_value(value, min, max))
}

fn stepped_value(
    value: i32,
    direction: NumberStep,
    step: i32,
    min: Option<i32>,
    max: Option<i32>,
) -> i32 {
    let value = match direction {
        NumberStep::Decrement => value.saturating_sub(step),
        NumberStep::Increment => value.saturating_add(step),
    };
    clamp_value(value, min, max)
}

fn clamp_value(value: i32, min: Option<i32>, max: Option<i32>) -> i32 {
    let value = min.map_or(value, |min| value.max(min));
    max.map_or(value, |max| value.min(max))
}

fn sync_text_binding(binding: &Binding<TextInputState>, cx: &mut App, value: i32) {
    let text = value.to_string();
    binding.update(cx, |state| {
        if state.value() == text {
            false
        } else {
            state.set_text(text);
            true
        }
    });
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn number_input_keeps_optional_suffix() {
        let input = NumberInput::new("number", 14).suffix("px");

        assert_eq!(input.suffix.as_deref(), Some("px"));
    }

    #[test]
    fn number_input_defaults_to_controls_around_value() {
        let input = NumberInput::new("number", 14);

        assert_eq!(input.layout, NumberInputLayout::ControlsAroundValue);
    }

    #[test]
    fn number_input_starts_read_only() {
        let input = NumberInput::new("number", 14);

        assert!(input.editable.is_none());
    }

    #[test]
    fn bound_number_input_stores_value_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| NumberInput::bound("number", cx.binding(14)));

        assert!(input.value_binding.is_some());
    }

    #[test]
    fn bound_number_input_can_store_editing_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| {
            NumberInput::bound("number", cx.binding(14)).input_bound(
                cx.focus_handle(),
                cx.binding(TextInputState::with_text("14")),
            )
        });

        assert!(
            input
                .editable
                .and_then(|editable| editable.binding)
                .is_some()
        );
    }

    #[test]
    fn clamp_value_respects_range() {
        assert_eq!(clamp_value(20, Some(11), Some(18)), 18);
    }

    #[test]
    fn stepped_value_saturates_and_clamps() {
        assert_eq!(
            stepped_value(i32::MAX, NumberStep::Increment, 4, None, Some(i32::MAX)),
            i32::MAX
        );
    }
}
