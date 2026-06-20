mod confirm;
mod panel;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, deferred, div,
    prelude::FluentBuilder, px,
};

use relay_ui_core::{
    icon::IconName,
    interaction::ClickHandler,
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
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    on_dismiss: Option<ClickHandler>,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            icon: None,
            width: 440.0,
            children: Vec::new(),
            footer: None,
            on_dismiss: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let on_dismiss = self.on_dismiss;
        let id = self.id.clone();
        let panel = DialogPanel {
            theme,
            id: self.id,
            title: self.title,
            description: self.description,
            icon: self.icon,
            width: self.width,
            children: self.children,
            footer: self.footer,
        }
        .render();

        deferred(
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
                .when_some(on_dismiss, |this, handler| {
                    this.cursor_pointer()
                        .on_click(move |event, window, cx| {
                            handler(event, window, cx);
                            cx.stop_propagation();
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                })
                .child(panel)
                .border_1()
                .border_color(theme.border),
        )
        .with_priority(theme::OVERLAY_PRIORITY_DIALOG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_width_can_be_overridden() {
        let dialog = Dialog::new("dialog", "Settings").width(520.0);

        assert_eq!(dialog.width, 520.0);
    }
}
