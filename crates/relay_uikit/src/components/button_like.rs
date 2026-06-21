use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use crate::{
    interaction::ClickHandler,
    theme::{BORDER_WIDTH, DISABLED_OPACITY, radius},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ButtonLikeColors {
    pub(crate) background: gpui::Hsla,
    pub(crate) border: gpui::Hsla,
    pub(crate) foreground: gpui::Hsla,
    pub(crate) hover_background: gpui::Hsla,
    pub(crate) hover_border: gpui::Hsla,
    pub(crate) hover_foreground: gpui::Hsla,
    pub(crate) active_background: gpui::Hsla,
    pub(crate) active_border: gpui::Hsla,
    pub(crate) active_foreground: gpui::Hsla,
}

impl ButtonLikeColors {
    pub(crate) fn new(
        background: gpui::Hsla,
        border: gpui::Hsla,
        foreground: gpui::Hsla,
        hover_background: gpui::Hsla,
        hover_border: gpui::Hsla,
        hover_foreground: gpui::Hsla,
    ) -> Self {
        Self {
            background,
            border,
            foreground,
            hover_background,
            hover_border,
            hover_foreground,
            active_background: hover_background.opacity(0.86),
            active_border: hover_border,
            active_foreground: hover_foreground,
        }
    }
}

/// Internal interactive button primitive shared by labelled and icon buttons.
#[derive(IntoElement)]
pub(crate) struct ButtonLike {
    id: ElementId,
    colors: ButtonLikeColors,
    width: Option<f32>,
    height: Option<f32>,
    padding_x: Option<f32>,
    gap: f32,
    text_size: Option<f32>,
    font_weight: Option<FontWeight>,
    disabled: bool,
    role: Role,
    on_click: Option<ClickHandler>,
    children: Vec<AnyElement>,
}

impl ButtonLike {
    pub(crate) fn new(id: impl Into<ElementId>, colors: ButtonLikeColors) -> Self {
        Self {
            id: id.into(),
            colors,
            width: None,
            height: None,
            padding_x: None,
            gap: 4.0,
            text_size: None,
            font_weight: None,
            disabled: false,
            role: Role::Button,
            on_click: None,
            children: Vec::new(),
        }
    }

    pub(crate) fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub(crate) fn size(mut self, size: f32) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    pub(crate) fn padding_x(mut self, padding: f32) -> Self {
        self.padding_x = Some(padding);
        self
    }

    pub(crate) fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub(crate) fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }

    pub(crate) fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn on_click(mut self, handler: Option<ClickHandler>) -> Self {
        self.on_click = handler;
        self
    }

    pub(crate) fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    fn interactive(&self) -> bool {
        self.on_click.is_some() && !self.disabled
    }
}

impl RenderOnce for ButtonLike {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let interactive = self.interactive();
        let colors = self.colors;
        let on_click = self.on_click.map(std::rc::Rc::new);

        div()
            .id(self.id)
            .border(px(BORDER_WIDTH))
            .rounded(px(radius::MD))
            .border_color(colors.border)
            .bg(colors.background)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(self.gap))
            .text_color(colors.foreground)
            .role(self.role)
            .tab_index(0)
            .when_some(self.width, |this, width| this.w(px(width)))
            .when_some(self.height, |this, height| this.h(px(height)))
            .when_some(self.padding_x, |this, padding| this.px(px(padding)))
            .when_some(self.text_size, |this, size| this.text_size(px(size)))
            .when_some(self.font_weight, |this, weight| this.font_weight(weight))
            .when(self.disabled, |this| this.opacity(DISABLED_OPACITY))
            .when(interactive, |this| {
                this.cursor_pointer()
                    .hover(move |style| {
                        style
                            .bg(colors.hover_background)
                            .border_color(colors.hover_border)
                            .text_color(colors.hover_foreground)
                    })
                    .active(move |style| {
                        style
                            .bg(colors.active_background)
                            .border_color(colors.active_border)
                            .text_color(colors.active_foreground)
                    })
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .children(self.children)
            .when_some(on_click.filter(|_| interactive), |this, handler| {
                let handler_for_key = handler.clone();
                this.on_click(move |event: &ClickEvent, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key.as_str() == " "
                        || event.keystroke.key.as_str() == "enter"
                    {
                        handler_for_key(&ClickEvent::default(), window, cx);
                        cx.stop_propagation();
                    }
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::rgb;

    use super::*;

    fn colors() -> ButtonLikeColors {
        ButtonLikeColors::new(
            rgb(0x000000).into(),
            rgb(0x111111).into(),
            rgb(0x222222).into(),
            rgb(0x333333).into(),
            rgb(0x444444).into(),
            rgb(0x555555).into(),
        )
    }

    #[test]
    fn button_like_starts_non_interactive() {
        let button = ButtonLike::new("button", colors());

        assert!(!button.interactive());
    }

    #[test]
    fn button_like_is_interactive_when_enabled_with_handler() {
        let button =
            ButtonLike::new("button", colors()).on_click(Some(Box::new(|_event, _window, _cx| {})));

        assert!(button.interactive());
    }

    #[test]
    fn button_like_disabled_suppresses_handler_interaction() {
        let button = ButtonLike::new("button", colors())
            .disabled(true)
            .on_click(Some(Box::new(|_event, _window, _cx| {})));

        assert!(!button.interactive());
    }

    #[test]
    fn button_like_colors_derive_active_from_hover_background() {
        let colors = colors();

        assert_eq!(
            colors.active_background,
            colors.hover_background.opacity(0.86)
        );
    }
}
