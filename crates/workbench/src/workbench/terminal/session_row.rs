use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use relay_foundation::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, BORDER_WIDTH, radius},
    tone::Tone,
};

use super::TerminalStatusBadge;

/// One row in the terminal/session history panel.
#[derive(IntoElement)]
pub struct TerminalSessionRow {
    id: ElementId,
    title: String,
    subtitle: String,
    status: Tone,
    active: bool,
    on_click: Option<ClickHandler>,
}

impl TerminalSessionRow {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: subtitle.into(),
            status: Tone::Muted,
            active: false,
            on_click: None,
        }
    }

    pub fn status(mut self, status: Tone) -> Self {
        self.status = status;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TerminalSessionRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_click;
        let clickable = handler.is_some();

        div()
            .id(self.id)
            .min_h(px(48.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if self.active {
                theme.accent_border
            } else {
                gpui::transparent_black()
            })
            .bg(if self.active {
                theme.accent_bg
            } else {
                gpui::transparent_black()
            })
            .when(clickable, |this| this.cursor_pointer())
            .when(clickable && !self.active, |this| {
                this.hover(move |style| style.bg(theme.hover))
            })
            .when(clickable, |this| {
                this.on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
            })
            .child(
                Icon::new(IconName::Terminal)
                    .size(IconSize::Small)
                    .color(self.status.fg(&theme)),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(BORDER_WIDTH))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(self.subtitle),
                    ),
            )
            .child(TerminalStatusBadge::new(self.status))
            .when_some(handler.filter(|_| clickable), |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
