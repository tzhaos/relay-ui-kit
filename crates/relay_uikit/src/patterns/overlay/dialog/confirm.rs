use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div,
};
use relay::Binding;

use crate::{
    button::{Button, ButtonVariant},
    icon::IconName,
    interaction::{OpenState, SharedClickHandler},
    theme::ActiveTheme,
};

use super::Dialog;

/// A common two-action confirmation dialog.
#[derive(IntoElement)]
pub struct ConfirmDialog {
    id: ElementId,
    title: String,
    description: String,
    confirm_label: String,
    cancel_label: String,
    open: bool,
    open_state: Option<OpenState>,
    focus_handle: Option<FocusHandle>,
    initial_focus: Option<FocusHandle>,
    danger: bool,
    on_confirm: Option<SharedClickHandler>,
    on_cancel: Option<SharedClickHandler>,
    on_dismiss: Option<SharedClickHandler>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmDialogInitialFocus {
    Confirm,
    Cancel,
}

impl ConfirmDialog {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            open: true,
            open_state: None,
            focus_handle: None,
            initial_focus: None,
            danger: false,
            on_confirm: None,
            on_cancel: None,
            on_dismiss: None,
        }
    }

    pub fn confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
        self
    }

    pub fn open_with(mut self, open_state: OpenState) -> Self {
        self.open_state = Some(open_state);
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn initial_focus(mut self, focus_handle: FocusHandle) -> Self {
        self.initial_focus = Some(focus_handle);
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    pub fn on_confirm(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm = Some(Rc::new(handler));
        self
    }

    pub fn on_cancel(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }

    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

fn default_initial_focus_target(danger: bool) -> ConfirmDialogInitialFocus {
    if danger {
        ConfirmDialogInitialFocus::Cancel
    } else {
        ConfirmDialogInitialFocus::Confirm
    }
}

impl RenderOnce for ConfirmDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            title,
            description,
            confirm_label,
            cancel_label,
            open,
            open_state,
            focus_handle,
            initial_focus,
            danger,
            on_confirm,
            on_cancel,
            on_dismiss,
        } = self;
        let dialog_focus = focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "dialog-focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let cancel_focus = window
            .use_keyed_state((id.clone(), "cancel-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let confirm_focus = window
            .use_keyed_state((id.clone(), "confirm-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let initial_focus =
            initial_focus.unwrap_or_else(|| match default_initial_focus_target(danger) {
                ConfirmDialogInitialFocus::Confirm => confirm_focus.clone(),
                ConfirmDialogInitialFocus::Cancel => cancel_focus.clone(),
            });
        let on_dismiss = on_dismiss.or_else(|| on_cancel.clone());
        let cancel = Button::new((id.clone(), "cancel"), cancel_label)
            .focus_handle(cancel_focus)
            .on_click({
                let on_cancel = on_cancel.clone();
                move |event, window, cx| {
                    if let Some(handler) = &on_cancel {
                        handler(event, window, cx);
                    }
                }
            });
        let confirm = Button::new((id.clone(), "confirm"), confirm_label)
            .focus_handle(confirm_focus)
            .variant(if danger {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            })
            .on_click(move |event, window, cx| {
                if let Some(handler) = &on_confirm {
                    handler(event, window, cx);
                }
            });

        let mut dialog = Dialog::new(id, title)
            .description(description)
            .icon(if danger {
                IconName::Archive
            } else {
                IconName::MessageSquareText
            })
            .open(open)
            .focus_handle(dialog_focus)
            .initial_focus(initial_focus)
            .footer(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("This action changes the active workspace state."),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(cancel)
                            .child(confirm),
                    ),
            );

        if let Some(open_state) = open_state {
            dialog = dialog.open_with(open_state);
        }

        if let Some(on_dismiss) = on_dismiss {
            dialog = dialog.on_dismiss(move |event, window, cx| {
                on_dismiss(event, window, cx);
            });
        }

        dialog
    }
}

#[cfg(test)]
mod tests {
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn confirm_dialog_defaults_to_safe_action() {
        let dialog = ConfirmDialog::new("confirm", "Close terminal", "Close this session?");

        assert!(!dialog.danger);
    }

    #[test]
    fn confirm_dialog_can_store_explicit_dismiss_handler() {
        let dialog = ConfirmDialog::new("confirm", "Close terminal", "Close this session?")
            .on_dismiss(|_event, _window, _cx| {});

        assert!(dialog.on_dismiss.is_some());
    }

    #[test]
    fn dangerous_confirm_dialog_prefers_cancel_as_initial_focus() {
        assert_eq!(
            default_initial_focus_target(true),
            ConfirmDialogInitialFocus::Cancel
        );
    }

    #[test]
    fn safe_confirm_dialog_prefers_confirm_as_initial_focus() {
        assert_eq!(
            default_initial_focus_target(false),
            ConfirmDialogInitialFocus::Confirm
        );
    }

    #[test]
    fn open_bound_confirm_dialog_stores_open_state() {
        let mut app = gpui::TestApp::new();
        let dialog = app.update(|cx| {
            ConfirmDialog::new("confirm", "Close terminal", "Close this session?")
                .open_bound(cx.binding(false))
        });

        assert!(dialog.open_state.is_some());
    }
}
