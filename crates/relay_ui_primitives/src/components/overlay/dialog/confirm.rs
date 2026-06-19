use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::{
    button::{Button, ButtonVariant},
    icon::IconName,
    interaction::SharedClickHandler,
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
    danger: bool,
    on_confirm: Option<SharedClickHandler>,
    on_cancel: Option<SharedClickHandler>,
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
            danger: false,
            on_confirm: None,
            on_cancel: None,
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
}

impl RenderOnce for ConfirmDialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let on_cancel = self.on_cancel;
        let on_confirm = self.on_confirm;
        let cancel = Button::new("confirm-cancel", self.cancel_label).on_click({
            let on_cancel = on_cancel.clone();
            move |event, window, cx| {
                if let Some(handler) = &on_cancel {
                    handler(event, window, cx);
                }
            }
        });
        let confirm = Button::new("confirm-primary", self.confirm_label)
            .variant(if self.danger {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            })
            .on_click(move |event, window, cx| {
                if let Some(handler) = &on_confirm {
                    handler(event, window, cx);
                }
            });

        Dialog::new(self.id, self.title)
            .description(self.description)
            .icon(if self.danger {
                IconName::Archive
            } else {
                IconName::MessageSquareText
            })
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
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_dialog_defaults_to_safe_action() {
        let dialog = ConfirmDialog::new("confirm", "Close terminal", "Close this session?");

        assert!(!dialog.danger);
    }
}
