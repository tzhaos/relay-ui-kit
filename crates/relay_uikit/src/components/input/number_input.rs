//! Compact numeric entry with optional text editing and stepper affordances.

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
    TextInputState,
    platform_input::{
        AfterEdit, PlatformInputBase, PlatformInputMode, PointerSelectionState,
        SingleLineInputProps, SingleLineInputStyle, single_line_input,
    },
    state::{InputActionKind, InputValueKind},
};

/// Layout strategy for the stepper controls around a [`NumberInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberInputLayout {
    /// Keep decrement and increment controls on opposite sides of the value.
    ControlsAroundValue,
    /// Group both controls on the trailing edge.
    ControlsTrailing,
}

#[derive(Clone, Copy)]
enum NumberStep {
    Decrement,
    Increment,
}

struct EditableNumberValue {
    focus: FocusHandle,
    binding: Binding<TextInputState>,
}

/// A compact Relay-bound numeric input with optional inline editing and stepper controls.
///
/// `NumberInput` keeps value ownership explicit:
///
/// - the numeric value always belongs to Relay through [`NumberInput::bound`];
/// - add [`NumberInput::input_bound`] when the host also wants a real
///   text-editing session for the rendered value.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    value_binding: Binding<i32>,
    suffix: Option<String>,
    layout: NumberInputLayout,
    editable: Option<EditableNumberValue>,
    input_focused: bool,
    input_key_context: String,
    input_on_key: Option<KeyHandler>,
    disabled: bool,
    min: Option<i32>,
    max: Option<i32>,
    step: i32,
    on_change: Option<SharedChangeHandler<i32>>,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
}

impl NumberInput {
    /// Create a Relay-bound numeric field backed by a [`Binding<i32>`].
    pub fn bound(id: impl Into<ElementId>, binding: Binding<i32>) -> Self {
        Self {
            id: id.into(),
            value_binding: binding,
            suffix: None,
            layout: NumberInputLayout::ControlsAroundValue,
            editable: None,
            input_focused: false,
            input_key_context: "NumberInput".into(),
            input_on_key: None,
            disabled: false,
            min: None,
            max: None,
            step: 1,
            on_change: None,
            on_decrement: None,
            on_increment: None,
        }
    }

    /// Render a non-editable suffix after the numeric value.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Choose whether stepper controls surround the value or trail it.
    pub fn layout(mut self, layout: NumberInputLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Disable the number input — blocks stepper buttons and keyboard input.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Clamp the value between an inclusive minimum and maximum.
    pub fn range(mut self, min: i32, max: i32) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    /// Set an inclusive minimum.
    pub fn min(mut self, min: i32) -> Self {
        self.min = Some(min);
        self
    }

    /// Set an inclusive maximum.
    pub fn max(mut self, max: i32) -> Self {
        self.max = Some(max);
        self
    }

    /// Override the step size used by the controls and keyboard handling.
    pub fn step(mut self, step: i32) -> Self {
        self.step = step.max(1);
        self
    }

    /// Enable Relay-bound inline editing driven by a [`Binding<TextInputState>`].
    pub fn input_bound(mut self, focus: FocusHandle, binding: Binding<TextInputState>) -> Self {
        self.editable = Some(EditableNumberValue { focus, binding });
        self
    }

    /// Force the focused visual treatment for the editable field.
    pub fn focused(mut self, focused: bool) -> Self {
        self.input_focused = focused;
        self
    }

    /// Override the GPUI key-dispatch context used while the editable field is focused.
    ///
    /// This accepts owned strings so higher-level product surfaces can derive
    /// editing contexts from runtime state.
    pub fn key_context(mut self, key_context: impl Into<String>) -> Self {
        self.input_key_context = key_context.into();
        self
    }

    /// Observe raw `keydown` events after Relay's built-in numeric editing behavior runs.
    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.input_on_key = Some(Box::new(handler));
        self
    }

    /// Observe committed numeric value changes from buttons or inline editing.
    pub fn on_change(mut self, handler: impl Fn(i32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    /// Observe decrement button activation after value updates are applied.
    pub fn on_decrement(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_decrement = Some(Box::new(handler));
        self
    }

    /// Observe increment button activation after value updates are applied.
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
            value_binding,
            suffix,
            layout,
            editable,
            input_focused,
            input_key_context,
            input_on_key,
            disabled,
            min,
            max,
            step,
            on_change,
            on_decrement,
            on_increment,
        } = self;
        let theme = *cx.theme();
        let pointer = editable.as_ref().map(|_| {
            window.use_keyed_state((id.clone(), "pointer-selection"), cx, |_, _| {
                PointerSelectionState::default()
            })
        });
        let current_value = value_binding.get(cx);
        let editable_text_binding = editable.as_ref().map(|editable| editable.binding.clone());
        let value_id = format!("{id}-value");
        let value = match (editable, pointer) {
            (Some(editable), Some(pointer)) => {
                let is_focused = input_focused || editable.focus.is_focused(window);
                let show_fallback = editable.binding.get(cx).is_empty() && !is_focused;
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
                    disabled,
                    is_focused,
                    show_fallback,
                    input_key_context,
                    input_on_key,
                    on_change.clone(),
                )
                .into_any_element()
            }
            (Some(_), None) | (None, _) => {
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
            disabled,
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
            disabled,
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
    pointer: gpui::Entity<PointerSelectionState>,
    fallback: i32,
    suffix: Option<String>,
    theme: crate::Theme,
    value_binding: Binding<i32>,
    min: Option<i32>,
    max: Option<i32>,
    disabled: bool,
    is_focused: bool,
    show_fallback: bool,
    key_context: String,
    on_key: Option<KeyHandler>,
    on_change: Option<SharedChangeHandler<i32>>,
) -> impl IntoElement {
    let id = id.into();
    let focus_for_click = editable.focus.clone();
    let focus_for_mouse_down = editable.focus.clone();
    let text_binding = editable.binding;
    let allow_negative = min.is_none_or(|min| min < 0);
    let handle_key = !disabled;
    let editor_style = SingleLineInputStyle {
        text_color: theme.text,
        placeholder_color: theme.text_muted,
        selection_color: theme.selection,
        cursor_color: theme.accent,
    };
    let after_edit = {
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
    };

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
        .key_context(key_context.as_str())
        .when(is_focused, |this| this.bg(theme.accent_bg))
        .when(disabled, |this| {
            this.opacity(DISABLED_OPACITY)
                .cursor(gpui::CursorStyle::OperationNotAllowed)
        })
        .when(!disabled, |this| this.cursor(gpui::CursorStyle::IBeam))
        .text_color(theme.text)
        .child(single_line_input(SingleLineInputProps {
            base: PlatformInputBase {
                id: (id.clone(), "editor").into(),
                focus: editable.focus.clone(),
                binding: text_binding.clone(),
                pointer,
                style: editor_style,
                placeholder: fallback.to_string(),
                show_placeholder: show_fallback,
                disabled,
            },
            mode: PlatformInputMode::Integer { allow_negative },
            after_edit: Some(after_edit),
        }))
        .when_some(suffix, |this, suffix| {
            this.child(div().text_xs().text_color(theme.text_muted).child(suffix))
        })
        .when(handle_key, |this| {
            this.on_key_down(move |event, window, cx| {
                let consumed = handle_bound_integer_platform_key(
                    &text_binding,
                    &value_binding,
                    &on_change,
                    event,
                    fallback,
                    min,
                    max,
                    window,
                    cx,
                );
                if let Some(on_key) = &on_key {
                    on_key(event, window, cx);
                }
                if consumed {
                    cx.stop_propagation();
                }
            })
        })
        .when(!disabled, |this| {
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                window.focus(&focus_for_mouse_down, cx);
            })
            .on_mouse_down_out(|_, window, _cx| {
                window.blur();
            })
            .on_click(move |_, window, cx| {
                window.focus(&focus_for_click, cx);
            })
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
    value_binding: Binding<i32>,
    text_binding: Option<Binding<TextInputState>>,
    on_change: Option<SharedChangeHandler<i32>>,
    direction: NumberStep,
    current_value: i32,
    min: Option<i32>,
    max: Option<i32>,
    step: i32,
    disabled: bool,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    let interactive = !disabled;

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
                        value_binding.set(cx, next);
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
    value_binding: &Binding<i32>,
    on_change: &Option<SharedChangeHandler<i32>>,
    event: &KeyDownEvent,
    current_value: i32,
    min: Option<i32>,
    max: Option<i32>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
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
            InputActionKind::Submit => {
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
    value_binding: &Binding<i32>,
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

    value_binding.set(cx, value);
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
        let mut app = TestApp::new();
        let input = app.update(|cx| NumberInput::bound("number", cx.binding(14)).suffix("px"));

        assert_eq!(input.suffix.as_deref(), Some("px"));
    }

    #[test]
    fn number_input_defaults_to_controls_around_value() {
        let mut app = TestApp::new();
        let input = app.update(|cx| NumberInput::bound("number", cx.binding(14)));

        assert_eq!(input.layout, NumberInputLayout::ControlsAroundValue);
    }

    #[test]
    fn number_input_starts_read_only() {
        let mut app = TestApp::new();
        let input = app.update(|cx| NumberInput::bound("number", cx.binding(14)));

        assert!(input.editable.is_none());
    }

    #[test]
    fn bound_number_input_stores_value_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| NumberInput::bound("number", cx.binding(14)));

        let value = app.read(|cx| input.value_binding.get(cx));
        assert_eq!(value, 14);
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

        let value = app.read(|cx| {
            input
                .editable
                .as_ref()
                .map(|editable| editable.binding.get(cx).value().to_string())
        });
        assert_eq!(value.as_deref(), Some("14"));
    }

    #[test]
    fn number_input_retains_key_context_set_before_input_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| {
            NumberInput::bound("number", cx.binding(14))
                .key_context(format!("NumberInput mode={}", "density"))
                .input_bound(
                    cx.focus_handle(),
                    cx.binding(TextInputState::with_text("14")),
                )
        });

        assert_eq!(input.input_key_context, "NumberInput mode=density");
    }

    #[test]
    fn number_input_retains_focus_override_set_before_input_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| {
            NumberInput::bound("number", cx.binding(14))
                .focused(true)
                .input_bound(
                    cx.focus_handle(),
                    cx.binding(TextInputState::with_text("14")),
                )
        });

        assert!(input.input_focused);
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
