use gpui::{
    App, Bounds, Corners, DispatchPhase, Edges, Element, ElementId, GlobalElementId,
    HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Position, Style, Styled, Window, div,
    point, px, quad, relative, size,
};

use super::state::{ScrollSurfaceState, schedule_scroll_decay};
use super::thumb::{ScrollThumbMetrics, THUMB_WIDTH};

const THUMB_INSET: f32 = 2.0;

pub(super) struct ScrollbarElement {
    id: ElementId,
    state: gpui::Entity<ScrollSurfaceState>,
    thumb: Option<ScrollThumbMetrics>,
    thumb_color: Hsla,
    rail_color: Hsla,
    thumb_opacity: f32,
}

pub(super) struct ScrollbarLayout {
    thumb_bounds: Option<Bounds<Pixels>>,
    thumb_hitbox: Option<gpui::Hitbox>,
}

impl ScrollbarElement {
    pub(super) fn new(
        id: ElementId,
        state: gpui::Entity<ScrollSurfaceState>,
        thumb: Option<ScrollThumbMetrics>,
        thumb_color: Hsla,
        rail_color: Hsla,
        thumb_opacity: f32,
    ) -> Self {
        Self {
            id,
            state,
            thumb,
            thumb_color,
            rail_color,
            thumb_opacity,
        }
    }
}

impl Element for ScrollbarElement {
    type RequestLayoutState = ();
    type PrepaintState = ScrollbarLayout;

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
        let style = Style {
            position: Position::Absolute,
            size: size(relative(1.0).into(), relative(1.0).into()),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let thumb_bounds = self.thumb.map(|thumb| {
            Bounds::new(
                point(
                    bounds.right() - px(THUMB_WIDTH + THUMB_INSET),
                    bounds.top() + px(thumb.top),
                ),
                size(px(THUMB_WIDTH), px(thumb.height)),
            )
        });
        let thumb_hitbox = thumb_bounds
            .map(|bounds| window.insert_hitbox(bounds, HitboxBehavior::BlockMouseExceptScroll));

        ScrollbarLayout {
            thumb_bounds,
            thumb_hitbox,
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
        let rail_bounds = Bounds::new(
            point(bounds.right() - px(1.0), bounds.top()),
            size(px(1.0), bounds.size.height),
        );
        window.paint_quad(gpui::fill(rail_bounds, self.rail_color));

        if let Some(thumb_bounds) = prepaint.thumb_bounds {
            window.paint_quad(quad(
                thumb_bounds,
                Corners::all(Pixels::MAX).clamp_radii_for_quad_size(thumb_bounds.size),
                self.thumb_color.opacity(self.thumb_opacity),
                Edges::default(),
                Hsla::transparent_black(),
                gpui::BorderStyle::default(),
            ));
        }

        let Some(thumb_hitbox) = prepaint.thumb_hitbox.clone() else {
            return;
        };
        let thumb_bounds = prepaint.thumb_bounds;
        let state_for_down = self.state.clone();
        let state_for_move = self.state.clone();
        let state_for_up = self.state.clone();
        let capture_phase = if self.state.read(cx).is_thumb_dragging() {
            DispatchPhase::Capture
        } else {
            DispatchPhase::Bubble
        };

        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase != capture_phase || event.button != MouseButton::Left {
                return;
            }

            let Some(thumb_bounds) = thumb_bounds else {
                return;
            };
            if !thumb_hitbox.is_hovered(window) {
                return;
            }

            let mouse_y = f32::from(event.position.y);
            let click_y = f32::from(event.position.y - thumb_bounds.top());
            let should_schedule_decay = state_for_down.update(cx, |state, cx| {
                state.start_thumb_drag(
                    mouse_y,
                    f32::from(thumb_bounds.top() - bounds.top()),
                    f32::from(thumb_bounds.size.height),
                    click_y,
                );
                state.update_thumb_drag(mouse_y);
                state.mark_scrolling();
                cx.notify();
                state.schedule_decay_if_needed()
            });
            if should_schedule_decay {
                schedule_scroll_decay(state_for_down.clone(), window);
            }
            cx.stop_propagation();
        });

        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != capture_phase {
                return;
            }

            let mouse_y = f32::from(event.position.y);
            let should_schedule_decay = state_for_move.update(cx, |state, cx| {
                if !state.is_thumb_dragging() || !event.dragging() {
                    return false;
                }
                let changed = state.update_thumb_drag(mouse_y);
                if changed {
                    state.mark_scrolling();
                    cx.notify();
                }
                changed && state.schedule_decay_if_needed()
            });
            if should_schedule_decay {
                schedule_scroll_decay(state_for_move.clone(), window);
            }
            cx.stop_propagation();
        });

        window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx| {
            if phase != capture_phase {
                return;
            }

            state_for_up.update(cx, |state, cx| {
                if state.is_thumb_dragging() {
                    state.end_thumb_drag();
                    cx.notify();
                }
            });
            cx.stop_propagation();
        });
    }
}

impl IntoElement for ScrollbarElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub(super) fn scrollbar_layer(
    id: ElementId,
    state: gpui::Entity<ScrollSurfaceState>,
    thumb: Option<ScrollThumbMetrics>,
    thumb_color: Hsla,
    rail_color: Hsla,
    thumb_opacity: f32,
) -> impl IntoElement {
    div().absolute().inset_0().child(ScrollbarElement::new(
        id,
        state,
        thumb,
        thumb_color,
        rail_color,
        thumb_opacity,
    ))
}
