mod confirm;
mod panel;

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window, deferred, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::IconName,
    interaction::{ClickHandler, OpenState},
    theme::{self, ActiveTheme, space},
};

use panel::DialogPanel;

pub use confirm::ConfirmDialog;

/// A centered dialog surface with an optional backdrop dismiss action.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    title: String,
    description: Option<String>,
    icon: Option<IconName>,
    width: f32,
    open: bool,
    open_state: Option<OpenState>,
    focus_handle: Option<FocusHandle>,
    initial_focus: Option<FocusHandle>,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    closable: bool,
    on_dismiss: Option<ClickHandler>,
}

#[derive(Default)]
struct DialogLifecycleState {
    was_open: bool,
    previous_focus: Option<FocusHandle>,
}

impl Dialog {
    /// Create a dialog with a stable id and visible title.
    pub fn new(id: impl Into<ElementId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            icon: None,
            width: 440.0,
            open: true,
            open_state: None,
            focus_handle: None,
            initial_focus: None,
            children: Vec::new(),
            footer: None,
            closable: true,
            on_dismiss: None,
        }
    }

    /// Add supporting copy under the title.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a title icon to reinforce the dialog intent.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Override the dialog panel width in pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Render the dialog open or closed from a host-owned snapshot.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Bind the dialog to shared Relay/host open state.
    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
        self
    }

    /// Supply an explicit shared open-state adapter.
    pub fn open_with(mut self, open_state: OpenState) -> Self {
        self.open_state = Some(open_state);
        self
    }

    /// Track focus on the dialog container with a host-owned focus handle.
    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    /// Redirect initial focus when the dialog opens.
    pub fn initial_focus(mut self, focus_handle: FocusHandle) -> Self {
        self.initial_focus = Some(focus_handle);
        self
    }

    /// Render a footer row, typically actions or status text.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Control whether backdrop click and `Escape` dismiss the dialog.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Observe dismiss requests after shared open-state cleanup runs.
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

fn schedule_dialog_focus(target: FocusHandle, window: &mut Window) {
    window.on_next_frame(move |window, _cx| {
        window.on_next_frame(move |window, cx| {
            window.focus(&target, cx);
        });
    });
}

fn sync_dialog_focus_state(
    state: &mut DialogLifecycleState,
    open: bool,
    container_focus: &FocusHandle,
    initial_focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) {
    if open && !state.was_open {
        state.previous_focus = window.focused(cx);
        schedule_dialog_focus(initial_focus.clone(), window);
    } else if !open && state.was_open {
        if let Some(previous_focus) = state.previous_focus.as_ref()
            && container_focus.contains_focused(window, cx)
        {
            window.focus(previous_focus, cx);
        }
        state.previous_focus = None;
    }

    state.was_open = open;
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            title,
            description,
            icon,
            width,
            open,
            open_state,
            focus_handle,
            initial_focus,
            children,
            footer,
            closable,
            on_dismiss,
        } = self;
        let dialog_focus = focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "focus-handle"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let initial_focus = initial_focus.unwrap_or_else(|| dialog_focus.clone());
        let open = open_state
            .as_ref()
            .map_or(open, |open_state| open_state.get(cx));
        let lifecycle = window.use_keyed_state((id.clone(), "lifecycle"), cx, |_, _| {
            DialogLifecycleState::default()
        });
        lifecycle.update(cx, |state, cx| {
            sync_dialog_focus_state(state, open, &dialog_focus, &initial_focus, window, cx);
        });

        let dismiss_handler = {
            let lifecycle = lifecycle.clone();
            let open_state = open_state.clone();
            let dialog_focus = dialog_focus.clone();
            let on_dismiss = on_dismiss.map(Rc::new);
            Rc::new(
                move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Some(open_state) = &open_state {
                        open_state.close(cx);
                    }
                    lifecycle.update(cx, |state, app| {
                        if let Some(previous_focus) = state.previous_focus.as_ref()
                            && dialog_focus.contains_focused(window, app)
                        {
                            window.focus(previous_focus, app);
                        }
                        state.previous_focus = None;
                        state.was_open = false;
                    });
                    if let Some(handler) = &on_dismiss {
                        handler(event, window, cx);
                    }
                },
            )
        };
        let panel = DialogPanel {
            theme,
            id: id.clone(),
            title,
            description,
            icon,
            width,
            focus_handle: dialog_focus,
            children,
            footer,
        }
        .render();

        let host = if open {
            div()
                .id((id, "backdrop"))
                .absolute()
                .inset_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(space::XL))
                .bg(gpui::black().opacity(0.22))
                .occlude()
                .when(closable, |this| {
                    let handler_for_click = dismiss_handler.clone();
                    let handler_for_key = dismiss_handler;
                    this.cursor_pointer()
                        .on_click(move |event, window, cx| {
                            handler_for_click(event, window, cx);
                            cx.stop_propagation();
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_key_down(move |event: &KeyDownEvent, window, cx| {
                            if event.keystroke.key.as_str() == "escape" {
                                handler_for_key(&ClickEvent::default(), window, cx);
                                cx.stop_propagation();
                            }
                        })
                })
                .child(panel)
                .border_1()
                .border_color(theme.border)
        } else {
            div().id((id, "dialog-host"))
        };

        deferred(host).with_priority(theme::OVERLAY_PRIORITY_DIALOG)
    }
}

#[cfg(test)]
mod tests {
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn dialog_width_can_be_overridden() {
        let dialog = Dialog::new("dialog", "Settings").width(520.0);

        assert_eq!(dialog.width, 520.0);
    }

    #[test]
    fn dialog_closable_defaults_to_true() {
        let dialog = Dialog::new("dialog", "Settings");

        assert!(dialog.closable);
    }

    #[test]
    fn dialog_open_builder_stores_value() {
        let dialog = Dialog::new("dialog", "Settings").open(false);

        assert!(!dialog.open);
    }

    #[test]
    fn dialog_open_bound_stores_open_state() {
        let mut app = gpui::TestApp::new();
        let dialog =
            app.update(|cx| Dialog::new("dialog", "Settings").open_bound(cx.binding(false)));

        assert!(dialog.open_state.is_some());
    }
}
