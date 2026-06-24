use std::{ops::Range, rc::Rc};

use gpui::{
    App, Bounds, ContentMask, DispatchPhase, Element, ElementId, FocusHandle, GlobalElementId,
    Hitbox, HitboxBehavior, Hsla, InputHandler, InspectorElementId, IntoElement, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ShapedLine, Style,
    TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window, fill, point, px, relative, size,
};
use relay::Binding;

use crate::theme::space;

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
const MULTILINE_LINE_GAP: f32 = space::XXS;
const SINGLE_LINE_SCROLL_INSET: f32 = 6.0;

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

pub(super) fn multiline_input(
    id: impl Into<ElementId>,
    focus: FocusHandle,
    binding: Binding<TextInputState>,
    pointer: gpui::Entity<PointerSelectionState>,
    style: SingleLineInputStyle,
    placeholder: impl Into<String>,
    show_placeholder: bool,
    disabled: bool,
    min_rows: usize,
) -> MultilineInputElement {
    MultilineInputElement {
        id: id.into(),
        focus,
        binding,
        pointer,
        style,
        placeholder: placeholder.into(),
        show_placeholder,
        disabled,
        min_rows: min_rows.max(1),
    }
}

pub(super) struct MultilineInputElement {
    id: ElementId,
    focus: FocusHandle,
    binding: Binding<TextInputState>,
    pointer: gpui::Entity<PointerSelectionState>,
    style: SingleLineInputStyle,
    placeholder: String,
    show_placeholder: bool,
    disabled: bool,
    min_rows: usize,
}

#[derive(Clone)]
struct MultilineLayoutLine {
    start: usize,
    content_end: usize,
    next_start: usize,
    y: Pixels,
    line: ShapedLine,
}

pub(super) struct MultilinePrepaintState {
    display_lines: Vec<(Point<Pixels>, ShapedLine)>,
    input_lines: Vec<MultilineLayoutLine>,
    selection_bounds: Vec<Bounds<Pixels>>,
    cursor_bounds: Option<Bounds<Pixels>>,
    hitbox: Hitbox,
    line_height: Pixels,
}

pub(super) struct SingleLinePrepaintState {
    display_line: ShapedLine,
    input_line: ShapedLine,
    selection_bounds: Option<Bounds<Pixels>>,
    cursor_bounds: Option<Bounds<Pixels>>,
    hitbox: Hitbox,
    text_origin: Point<Pixels>,
    scroll_offset: Pixels,
}

impl IntoElement for SingleLineInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl IntoElement for MultilineInputElement {
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
        let input_line = window.text_system().shape_line(
            state.value().to_string().into(),
            font_size,
            &[input_run],
            None,
        );

        let display_line = shape_display_line(
            &state,
            &self.placeholder,
            self.show_placeholder,
            self.style,
            window,
            font_size,
        );
        let scroll_offset = single_line_scroll_offset(&state, &input_line, bounds.size.width);

        let selection_bounds = state.selection_range().map(|(start, end)| {
            Bounds::from_corners(
                point(
                    bounds.left() + input_line.x_for_index(start) - scroll_offset,
                    bounds.top(),
                ),
                point(
                    bounds.left() + input_line.x_for_index(end) - scroll_offset,
                    bounds.bottom(),
                ),
            )
        });

        let cursor_bounds = if selection_bounds.is_none() {
            let cursor_x = input_line.x_for_index(state.cursor());
            Some(Bounds::new(
                point(bounds.left() + cursor_x - scroll_offset, bounds.top()),
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
            text_origin: point(bounds.left() - scroll_offset, bounds.top()),
            scroll_offset,
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
                    prepaint.scroll_offset,
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
        let scroll_offset = prepaint.scroll_offset;
        let disabled = self.disabled;

        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if disabled || phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                return;
            }
            if !hitbox_for_down.is_hovered(window) {
                return;
            }

            let index = closest_index_for_mouse(
                bounds,
                &input_line_for_down,
                scroll_offset,
                event.position,
            );
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

            let index = closest_index_for_mouse(
                bounds,
                &input_line_for_move,
                scroll_offset,
                event.position,
            );
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

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection_bounds) = prepaint.selection_bounds.take() {
                window.paint_quad(fill(selection_bounds, self.style.selection_color));
            }

            prepaint
                .display_line
                .paint(
                    prepaint.text_origin,
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
        });
    }
}

impl Element for MultilineInputElement {
    type RequestLayoutState = ();
    type PrepaintState = MultilinePrepaintState;

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
        let line_height = window.line_height();
        let line_count = line_segments(self.binding.get(cx).value())
            .len()
            .max(self.min_rows);
        let height = line_height * line_count as f32
            + px(MULTILINE_LINE_GAP) * (line_count.saturating_sub(1) as f32);

        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = height.into();
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
        let line_gap = px(MULTILINE_LINE_GAP);

        let input_lines = build_multiline_layout_lines(
            &state,
            window,
            font_size,
            line_height,
            line_gap,
            self.min_rows,
            self.style.text_color,
        );

        let display_lines = if state.value().is_empty() && self.show_placeholder {
            vec![(
                bounds.origin,
                shape_plain_line(
                    &self.placeholder,
                    self.style.placeholder_color,
                    window,
                    font_size,
                ),
            )]
        } else {
            input_lines
                .iter()
                .map(|line| {
                    (
                        point(bounds.left(), bounds.top() + line.y),
                        line.line.clone(),
                    )
                })
                .collect()
        };

        let selection_bounds =
            selection_bounds_for_lines(&state, &input_lines, bounds, line_height);
        let cursor_bounds = if selection_bounds.is_empty() {
            let cursor = cursor_bounds_for_lines(&state, &input_lines, bounds, line_height);
            cursor
        } else {
            None
        };

        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        MultilinePrepaintState {
            display_lines,
            input_lines,
            selection_bounds,
            cursor_bounds,
            hitbox,
            line_height,
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
                MultilinePlatformInputHandler::new(
                    self.binding.clone(),
                    prepaint.input_lines.clone(),
                    bounds,
                    prepaint.line_height,
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
        let lines_for_down = prepaint.input_lines.clone();
        let lines_for_move = prepaint.input_lines.clone();
        let line_height = prepaint.line_height;
        let disabled = self.disabled;

        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if disabled || phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                return;
            }
            if !hitbox_for_down.is_hovered(window) {
                return;
            }

            let index = closest_index_for_multiline_mouse(
                bounds,
                &lines_for_down,
                line_height,
                event.position,
            );
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

            let index = closest_index_for_multiline_mouse(
                bounds,
                &lines_for_move,
                line_height,
                event.position,
            );
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

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for selection_bounds in prepaint.selection_bounds.drain(..) {
                window.paint_quad(fill(selection_bounds, self.style.selection_color));
            }

            for (origin, line) in prepaint.display_lines.drain(..) {
                line.paint(origin, line_height, TextAlign::Left, None, window, cx)
                    .ok();
            }

            if self.focus.is_focused(window)
                && let Some(cursor_bounds) = prepaint.cursor_bounds.take()
            {
                window.paint_quad(fill(cursor_bounds, self.style.cursor_color));
            }
        });
    }
}

#[derive(Clone)]
struct SingleLinePlatformInputHandler {
    binding: Binding<TextInputState>,
    bounds: Bounds<Pixels>,
    line: ShapedLine,
    scroll_offset: Pixels,
    mode: PlatformInputMode,
    after_edit: Option<AfterEdit>,
}

#[derive(Clone)]
struct MultilinePlatformInputHandler {
    binding: Binding<TextInputState>,
    lines: Vec<MultilineLayoutLine>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
}

impl MultilinePlatformInputHandler {
    fn new(
        binding: Binding<TextInputState>,
        lines: Vec<MultilineLayoutLine>,
        bounds: Bounds<Pixels>,
        line_height: Pixels,
    ) -> Self {
        Self {
            binding,
            lines,
            bounds,
            line_height,
        }
    }
}

impl SingleLinePlatformInputHandler {
    fn new(
        binding: Binding<TextInputState>,
        bounds: Bounds<Pixels>,
        line: ShapedLine,
        scroll_offset: Pixels,
        mode: PlatformInputMode,
        after_edit: Option<AfterEdit>,
    ) -> Self {
        Self {
            binding,
            bounds,
            line,
            scroll_offset,
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

    fn marked_text_range(&mut self, _window: &mut Window, cx: &mut App) -> Option<Range<usize>> {
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
        self.apply_edit(
            replacement_range,
            text,
            window,
            cx,
            |state, replacement_range, text| {
                state.replace_text_in_range_utf16(replacement_range, text);
            },
        );
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.apply_edit(
            range_utf16,
            new_text,
            window,
            cx,
            |state, range_utf16, new_text| {
                state.replace_and_mark_text_in_range_utf16(
                    range_utf16,
                    new_text,
                    new_selected_range,
                );
            },
        );
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
                self.bounds.left() + self.line.x_for_index(range.start) - self.scroll_offset,
                self.bounds.top(),
            ),
            point(
                self.bounds.left() + self.line.x_for_index(range.end) - self.scroll_offset,
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
        let byte_index = self
            .line
            .closest_index_for_x(local_point.x + self.scroll_offset);
        Some(self.binding.get(cx).utf16_offset_for_byte(byte_index))
    }

    fn prefers_ime_for_printable_keys(&mut self, _window: &mut Window, _cx: &mut App) -> bool {
        true
    }
}

impl InputHandler for MultilinePlatformInputHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<UTF16Selection> {
        Some(self.binding.get(cx).selected_text_range_utf16())
    }

    fn marked_text_range(&mut self, _window: &mut Window, cx: &mut App) -> Option<Range<usize>> {
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
        _window: &mut Window,
        cx: &mut App,
    ) {
        let text = normalize_multiline_text(text);
        self.binding.update(cx, |state| {
            state.replace_text_in_range_utf16(replacement_range, &text);
            true
        });
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) {
        let text = normalize_multiline_text(new_text);
        self.binding.update(cx, |state| {
            state.replace_and_mark_text_in_range_utf16(range_utf16, &text, new_selected_range);
            true
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
        let byte_range = state.byte_range_for_utf16(&range_utf16);
        let (start_idx, start_local) = line_position_for_offset(&self.lines, byte_range.start)?;
        let (end_idx, end_local) = line_position_for_offset(&self.lines, byte_range.end)?;
        let start_line = &self.lines[start_idx];
        let end_line = &self.lines[end_idx];
        let top = self.bounds.top() + start_line.y;

        let left = self.bounds.left() + start_line.line.x_for_index(start_local);
        let right = if start_idx == end_idx {
            self.bounds.left() + end_line.line.x_for_index(end_local)
        } else {
            self.bounds.left() + start_line.line.width()
        };

        Some(Bounds::from_corners(
            point(left, top),
            point(right.max(left), top + self.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        let byte_index =
            closest_index_for_multiline_mouse(self.bounds, &self.lines, self.line_height, point);
        Some(self.binding.get(cx).utf16_offset_for_byte(byte_index))
    }

    fn prefers_ime_for_printable_keys(&mut self, _window: &mut Window, _cx: &mut App) -> bool {
        true
    }
}

fn closest_index_for_mouse(
    bounds: Bounds<Pixels>,
    line: &ShapedLine,
    scroll_offset: Pixels,
    position: Point<Pixels>,
) -> usize {
    if position.x <= bounds.left() {
        return 0;
    }
    if position.x >= bounds.right() {
        return line.len();
    }
    line.closest_index_for_x(position.x - bounds.left() + scroll_offset)
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
        return window.text_system().shape_line(
            placeholder.to_string().into(),
            font_size,
            &[run],
            None,
        );
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

fn single_line_scroll_offset(
    state: &TextInputState,
    line: &ShapedLine,
    available_width: Pixels,
) -> Pixels {
    let max_offset = (line.width() - available_width).max(px(0.0));
    if max_offset == px(0.0) {
        return px(0.0);
    }

    let focus_index = state
        .marked_range()
        .map_or_else(|| state.cursor(), |range| range.end);
    let focus_x = line.x_for_index(focus_index);
    let desired_offset = focus_x - available_width + px(SINGLE_LINE_SCROLL_INSET);
    desired_offset.clamp(px(0.0), max_offset)
}

fn line_segments(value: &str) -> Vec<(usize, usize, usize, String)> {
    if value.is_empty() {
        return vec![(0, 0, 0, String::new())];
    }

    let mut lines = Vec::new();
    let mut start = 0;
    for segment in value.split_inclusive('\n') {
        let display = segment.strip_suffix('\n').unwrap_or(segment);
        let content_end = start + display.len();
        let next_start = start + segment.len();
        lines.push((start, content_end, next_start, display.to_string()));
        start = next_start;
    }

    if value.ends_with('\n') {
        let len = value.len();
        lines.push((len, len, len, String::new()));
    }

    lines
}

fn build_multiline_layout_lines(
    state: &TextInputState,
    window: &Window,
    font_size: Pixels,
    line_height: Pixels,
    line_gap: Pixels,
    min_rows: usize,
    text_color: Hsla,
) -> Vec<MultilineLayoutLine> {
    let segments = line_segments(state.value());
    let total_rows = segments.len().max(min_rows);
    let mut lines = Vec::with_capacity(total_rows);

    for (index, (start, content_end, next_start, text)) in segments.into_iter().enumerate() {
        let y = (line_height + line_gap) * index as f32;
        lines.push(MultilineLayoutLine {
            start,
            content_end,
            next_start,
            y,
            line: shape_marked_multiline_line(
                &text,
                start,
                state.marked_range(),
                text_color,
                window,
                font_size,
            ),
        });
    }

    for index in lines.len()..total_rows {
        let y = (line_height + line_gap) * index as f32;
        lines.push(MultilineLayoutLine {
            start: state.value().len(),
            content_end: state.value().len(),
            next_start: state.value().len(),
            y,
            line: shape_plain_line("", text_color, window, font_size),
        });
    }

    lines
}

fn shape_plain_line(text: &str, color: Hsla, window: &Window, font_size: Pixels) -> ShapedLine {
    let mut run = window.text_style().to_run(text.len());
    run.color = color;
    window
        .text_system()
        .shape_line(text.to_string().into(), font_size, &[run], None)
}

fn shape_marked_multiline_line(
    text: &str,
    line_start: usize,
    marked_range: Option<Range<usize>>,
    color: Hsla,
    window: &Window,
    font_size: Pixels,
) -> ShapedLine {
    let mut base_run = window.text_style().to_run(text.len());
    base_run.color = color;

    let Some(marked_range) = marked_range else {
        return window.text_system().shape_line(
            text.to_string().into(),
            font_size,
            &[base_run],
            None,
        );
    };

    let local_start = marked_range
        .start
        .saturating_sub(line_start)
        .min(text.len());
    let local_end = marked_range.end.saturating_sub(line_start).min(text.len());
    if local_start >= local_end {
        return window.text_system().shape_line(
            text.to_string().into(),
            font_size,
            &[base_run],
            None,
        );
    }

    let runs = vec![
        TextRun {
            len: local_start,
            ..base_run.clone()
        },
        TextRun {
            len: local_end - local_start,
            underline: Some(UnderlineStyle {
                color: Some(color),
                thickness: px(1.0),
                wavy: false,
            }),
            ..base_run.clone()
        },
        TextRun {
            len: text.len() - local_end,
            ..base_run
        },
    ]
    .into_iter()
    .filter(|run| run.len > 0)
    .collect::<Vec<_>>();

    window
        .text_system()
        .shape_line(text.to_string().into(), font_size, &runs, None)
}

fn selection_bounds_for_lines(
    state: &TextInputState,
    lines: &[MultilineLayoutLine],
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Vec<Bounds<Pixels>> {
    let Some((selection_start, selection_end)) = state.selection_range() else {
        return Vec::new();
    };

    lines
        .iter()
        .filter_map(|line| {
            let visible_start = selection_start.max(line.start).min(line.content_end);
            let visible_end = selection_end.max(line.start).min(line.content_end);
            (visible_start < visible_end).then(|| {
                Bounds::from_corners(
                    point(
                        bounds.left() + line.line.x_for_index(visible_start - line.start),
                        bounds.top() + line.y,
                    ),
                    point(
                        bounds.left() + line.line.x_for_index(visible_end - line.start),
                        bounds.top() + line.y + line_height,
                    ),
                )
            })
        })
        .collect()
}

fn cursor_bounds_for_lines(
    state: &TextInputState,
    lines: &[MultilineLayoutLine],
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Option<Bounds<Pixels>> {
    let (line_index, local_byte) = line_position_for_offset(lines, state.cursor())?;
    let line = &lines[line_index];
    let cursor_x = line.line.x_for_index(local_byte);
    Some(Bounds::new(
        point(bounds.left() + cursor_x, bounds.top() + line.y),
        size(px(1.5), line_height),
    ))
}

fn line_position_for_offset(
    lines: &[MultilineLayoutLine],
    offset: usize,
) -> Option<(usize, usize)> {
    for (index, line) in lines.iter().enumerate() {
        if offset < line.next_start {
            let local = offset
                .saturating_sub(line.start)
                .min(line.content_end - line.start);
            return Some((index, local));
        }
    }

    lines.last().map(|line| {
        (
            lines.len().saturating_sub(1),
            line.content_end.saturating_sub(line.start),
        )
    })
}

fn closest_index_for_multiline_mouse(
    bounds: Bounds<Pixels>,
    lines: &[MultilineLayoutLine],
    line_height: Pixels,
    position: Point<Pixels>,
) -> usize {
    if lines.is_empty() {
        return 0;
    }

    let local_y = position.y - bounds.top();
    let mut chosen_index = lines.len().saturating_sub(1);
    for (index, line) in lines.iter().enumerate() {
        let threshold = line.y + line_height + px(MULTILINE_LINE_GAP / 2.0);
        if local_y < threshold {
            chosen_index = index;
            break;
        }
    }

    let line = &lines[chosen_index];
    let local_byte = if position.x <= bounds.left() {
        0
    } else if position.x >= bounds.right() {
        line.content_end.saturating_sub(line.start)
    } else {
        line.line.closest_index_for_x(position.x - bounds.left())
    };

    (line.start + local_byte).min(line.content_end)
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

fn normalize_multiline_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_multiline_text_coalesces_crlf_and_cr() {
        assert_eq!(
            normalize_multiline_text("one\r\ntwo\rthree"),
            "one\ntwo\nthree"
        );
    }

    #[test]
    fn line_segments_preserve_trailing_empty_line() {
        let segments = line_segments("alpha\nbeta\n");

        assert_eq!(
            segments,
            vec![
                (0, 5, 6, "alpha".to_string()),
                (6, 10, 11, "beta".to_string()),
                (11, 11, 11, String::new()),
            ]
        );
    }

    #[test]
    fn line_position_for_offset_maps_trailing_newline_to_empty_row() {
        let lines = vec![
            MultilineLayoutLine {
                start: 0,
                content_end: 5,
                next_start: 6,
                y: px(0.0),
                line: ShapedLine::default(),
            },
            MultilineLayoutLine {
                start: 6,
                content_end: 6,
                next_start: 6,
                y: px(22.0),
                line: ShapedLine::default(),
            },
        ];

        assert_eq!(line_position_for_offset(&lines, 0), Some((0, 0)));
        assert_eq!(line_position_for_offset(&lines, 5), Some((0, 5)));
        assert_eq!(line_position_for_offset(&lines, 6), Some((1, 0)));
        assert_eq!(line_position_for_offset(&lines, 10), Some((1, 0)));
    }
}
