use std::{ops::Range, rc::Rc};

use gpui::{
    App, Bounds, DispatchPhase, Element, ElementId, FocusHandle, GlobalElementId, Hitbox,
    HitboxBehavior, Hsla, InputHandler, InspectorElementId, IntoElement, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ShapedLine, Style, TextAlign,
    TextRun, UTF16Selection, UnderlineStyle, Window, fill, point, px, relative, size,
};
use relay::Binding;

use super::TextInputState;

#[derive(Clone, Copy)]
pub(super) struct SingleLineInputStyle {
    pub text_color: Hsla,
    pub placeholder_color: Hsla,
    pub selection_color: Hsla,
    pub cursor_color: Hsla,
}

#[derive(Clone, Copy)]
pub(super) enum PlatformInputMode {
    Text,
    Integer { allow_negative: bool },
}

#[derive(Default)]
pub(super) struct PointerSelectionState {
    selecting: bool,
}

pub(super) type AfterEdit = Rc<dyn Fn(&Binding<TextInputState>, &mut Window, &mut App) + 'static>;

pub(super) fn single_line_input(
    id: impl Into<ElementId>,
    focus: FocusHandle,
    binding: Binding<TextInputState>,
    pointer: gpui::Entity<PointerSelectionState>,
    style: SingleLineInputStyle,
    placeholder: impl Into<String>,
    show_placeholder: bool,
    disabled: bool,
    mode: PlatformInputMode,
    after_edit: Option<AfterEdit>,
) -> SingleLineInputElement {
    SingleLineInputElement {
        id: id.into(),
        focus,
        binding,
        pointer,
        style,
        placeholder: placeholder.into(),
        show_placeholder,
        disabled,
        mode,
        after_edit,
    }
}

pub(super) struct SingleLineInputElement {
    id: ElementId,
    focus: FocusHandle,
    binding: Binding<TextInputState>,
    pointer: gpui::Entity<PointerSelectionState>,
    style: SingleLineInputStyle,
    placeholder: String,
    show_placeholder: bool,
    disabled: bool,
    mode: PlatformInputMode,
    after_edit: Option<AfterEdit>,
}

pub(super) struct SingleLinePrepaintState {
    display_line: ShapedLine,
    input_line: ShapedLine,
    selection_bounds: Option<Bounds<Pixels>>,
    cursor_bounds: Option<Bounds<Pixels>>,
    hitbox: Hitbox,
}

impl IntoElement for SingleLineInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SingleLineInputElement {
    type RequestLayoutState = ();
    type PrepaintState = SingleLinePrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let state = self.binding.get(cx);
        let text_style = window.text_style();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let input_run = text_style.to_run(state.value().len());
        let input_line = window
            .text_system()
            .shape_line(state.value().to_string().into(), font_size, &[input_run], None);

        let display_line = shape_display_line(
            &state,
            &self.placeholder,
            self.show_placeholder,
            self.style,
            window,
            font_size,
        );

        let selection_bounds = state.selection_range().map(|(start, end)| {
            Bounds::from_corners(
                point(bounds.left() + input_line.x_for_index(start), bounds.top()),
                point(bounds.left() + input_line.x_for_index(end), bounds.bottom()),
            )
        });

        let cursor_bounds = if selection_bounds.is_none() {
            let cursor_x = input_line.x_for_index(state.cursor());
            Some(Bounds::new(
                point(bounds.left() + cursor_x, bounds.top()),
                size(px(1.5), line_height),
            ))
        } else {
            None
        };

        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        SingleLinePrepaintState {
            display_line,
            input_line,
            selection_bounds,
            cursor_bounds,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !self.disabled {
            window.handle_input(
                &self.focus,
                SingleLinePlatformInputHandler::new(
                    self.binding.clone(),
                    bounds,
                    prepaint.input_line.clone(),
                    self.mode,
                    self.after_edit.clone(),
                ),
                cx,
            );
        }

        let hitbox_for_down = prepaint.hitbox.clone();
        let hitbox_for_move = prepaint.hitbox.clone();
        let binding_for_down = self.binding.clone();
        let binding_for_move = self.binding.clone();
        let pointer_for_down = self.pointer.clone();
        let pointer_for_move = self.pointer.clone();
        let pointer_for_up = self.pointer.clone();
        let focus_for_down = self.focus.clone();
        let input_line_for_down = prepaint.input_line.clone();
        let input_line_for_move = prepaint.input_line.clone();
        let disabled = self.disabled;

        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if disabled || phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                return;
            }
            if !hitbox_for_down.is_hovered(window) {
                return;
            }

            let index = closest_index_for_mouse(bounds, &input_line_for_down, event.position);
            binding_for_down.update(cx, |state| {
                if event.modifiers.shift {
                    state.extend_selection_to(index);
                } else {
                    state.set_cursor(index);
                }
                true
            });
            pointer_for_down.update(cx, |pointer, _cx| {
                pointer.selecting = true;
            });
            window.focus(&focus_for_down, cx);
            window.prevent_default();
            cx.stop_propagation();
        });

        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if disabled || phase != DispatchPhase::Bubble {
                return;
            }
            if !pointer_for_move.read(cx).selecting || !event.dragging() {
                return;
            }
            if !hitbox_for_move.is_hovered(window) && !bounds.contains(&event.position) {
                return;
            }

            let index = closest_index_for_mouse(bounds, &input_line_for_move, event.position);
            binding_for_move.update(cx, |state| {
                state.extend_selection_to(index);
                true
            });
            cx.stop_propagation();
        });

        window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx| {
            if phase != DispatchPhase::Bubble {
                return;
            }
            let should_stop = pointer_for_up.update(cx, |pointer, _cx| {
                let was_selecting = pointer.selecting;
                pointer.selecting = false;
                was_selecting
            });
            if should_stop {
                cx.stop_propagation();
            }
        });

        if let Some(selection_bounds) = prepaint.selection_bounds.take() {
            window.paint_quad(fill(selection_bounds, self.style.selection_color));
        }

        prepaint
            .display_line
            .paint(
                bounds.origin,
                window.line_height(),
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();

        if self.focus.is_focused(window)
            && let Some(cursor_bounds) = prepaint.cursor_bounds.take()
        {
            window.paint_quad(fill(cursor_bounds, self.style.cursor_color));
        }
    }
}

#[derive(Clone)]
struct SingleLinePlatformInputHandler {
    binding: Binding<TextInputState>,
    bounds: Bounds<Pixels>,
    line: ShapedLine,
    mode: PlatformInputMode,
    after_edit: Option<AfterEdit>,
}

impl SingleLinePlatformInputHandler {
    fn new(
        binding: Binding<TextInputState>,
        bounds: Bounds<Pixels>,
        line: ShapedLine,
        mode: PlatformInputMode,
        after_edit: Option<AfterEdit>,
    ) -> Self {
        Self {
            binding,
            bounds,
            line,
            mode,
            after_edit,
        }
    }

    fn apply_edit(
        &self,
        replacement_range: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut App,
        mutate: impl FnOnce(&mut TextInputState, Option<Range<usize>>, &str),
    ) {
        let Some(new_text) = normalize_input_text(
            self.mode,
            &self.binding.get(cx),
            replacement_range.clone(),
            new_text,
        ) else {
            return;
        };

        self.binding.update(cx, |state| {
            mutate(state, replacement_range, &new_text);
            true
        });
        if let Some(after_edit) = &self.after_edit {
            after_edit(&self.binding, window, cx);
        }
    }
}

impl InputHandler for SingleLinePlatformInputHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<UTF16Selection> {
        Some(self.binding.get(cx).selected_text_range_utf16())
    }

    fn marked_text_range(
        &mut self,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<Range<usize>> {
        self.binding.get(cx).marked_text_range_utf16()
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        self.binding
            .get(cx)
            .text_for_range_utf16(range_utf16, adjusted_range)
    }

    fn replace_text_in_range(
        &mut self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.apply_edit(replacement_range, text, window, cx, |state, replacement_range, text| {
            state.replace_text_in_range_utf16(replacement_range, text);
        });
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.apply_edit(range_utf16, new_text, window, cx, |state, range_utf16, new_text| {
            state.replace_and_mark_text_in_range_utf16(
                range_utf16,
                new_text,
                new_selected_range,
            );
        });
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut App) {
        self.binding.update(cx, |state| {
            state.unmark_text();
            true
        });
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        let state = self.binding.get(cx);
        let range = state.byte_range_for_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                self.bounds.left() + self.line.x_for_index(range.start),
                self.bounds.top(),
            ),
            point(
                self.bounds.left() + self.line.x_for_index(range.end),
                self.bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        let local_point = self.bounds.localize(&point)?;
        let byte_index = self.line.index_for_x(local_point.x)?;
        Some(self.binding.get(cx).utf16_offset_for_byte(byte_index))
    }
}

fn closest_index_for_mouse(bounds: Bounds<Pixels>, line: &ShapedLine, position: Point<Pixels>) -> usize {
    if position.x <= bounds.left() {
        return 0;
    }
    if position.x >= bounds.right() {
        return line.len();
    }
    line.closest_index_for_x(position.x - bounds.left())
}

fn shape_display_line(
    state: &TextInputState,
    placeholder: &str,
    show_placeholder: bool,
    style: SingleLineInputStyle,
    window: &Window,
    font_size: Pixels,
) -> ShapedLine {
    let text_style = window.text_style();

    if state.value().is_empty() && show_placeholder {
        let mut run = text_style.to_run(placeholder.len());
        run.color = style.placeholder_color;
        return window
            .text_system()
            .shape_line(placeholder.to_string().into(), font_size, &[run], None);
    }

    let mut base_run = text_style.to_run(state.value().len());
    base_run.color = style.text_color;
    let runs = if let Some(marked_range) = state.marked_range() {
        vec![
            TextRun {
                len: marked_range.start,
                ..base_run.clone()
            },
            TextRun {
                len: marked_range.end - marked_range.start,
                underline: Some(UnderlineStyle {
                    color: Some(style.text_color),
                    thickness: px(1.0),
                    wavy: false,
                }),
                ..base_run.clone()
            },
            TextRun {
                len: state.value().len() - marked_range.end,
                ..base_run
            },
        ]
        .into_iter()
        .filter(|run| run.len > 0)
        .collect::<Vec<_>>()
    } else {
        vec![base_run]
    };

    window
        .text_system()
        .shape_line(state.value().to_string().into(), font_size, &runs, None)
}

fn normalize_input_text(
    mode: PlatformInputMode,
    state: &TextInputState,
    replacement_range: Option<Range<usize>>,
    new_text: &str,
) -> Option<String> {
    match mode {
        PlatformInputMode::Text => Some(new_text.replace('\n', " ")),
        PlatformInputMode::Integer { allow_negative } => {
            sanitize_integer_text(state, replacement_range, new_text, allow_negative)
        }
    }
}

fn sanitize_integer_text(
    state: &TextInputState,
    replacement_range: Option<Range<usize>>,
    new_text: &str,
    allow_negative: bool,
) -> Option<String> {
    if new_text.is_empty() {
        return Some(String::new());
    }

    let filtered = new_text
        .chars()
        .filter(|c| c.is_ascii_digit() || (allow_negative && *c == '-'))
        .collect::<String>();
    if filtered.is_empty() {
        return None;
    }

    let mut preview = state.clone();
    preview.replace_text_in_range_utf16(replacement_range, &filtered);
    is_valid_partial_integer(preview.value(), allow_negative).then_some(filtered)
}

fn is_valid_partial_integer(value: &str, allow_negative: bool) -> bool {
    if value.is_empty() {
        return true;
    }
    if allow_negative && value == "-" {
        return true;
    }

    let digits = if allow_negative {
        match value.strip_prefix('-') {
            Some(rest) => rest,
            None => value,
        }
    } else {
        value
    };

    !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
        && value.matches('-').count() <= usize::from(allow_negative)
        && (!value.contains('-') || value.starts_with('-'))
}
