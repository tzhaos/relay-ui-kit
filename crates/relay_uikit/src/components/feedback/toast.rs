use std::time::Duration;

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use crate::{
    components::button::IconButton,
    icon::{IconName, IconSize},
    interaction::ClickHandler,
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, radius},
    tone::Tone,
};

use super::shared::{feedback_icon, tone_icon};

/// A compact floating notification body.
#[derive(IntoElement)]
pub struct Toast {
    id: ElementId,
    title: String,
    detail: Option<String>,
    tone: Tone,
    animated: bool,
    duration: Option<Duration>,
    on_close: Option<ClickHandler>,
    on_dismiss: Option<ClickHandler>,
}

impl Toast {
    pub fn new(id: impl Into<ElementId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: None,
            tone: Tone::Info,
            animated: true,
            duration: None,
            on_close: None,
            on_dismiss: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let close_id: ElementId = (self.id.clone(), "close").into();
        let on_close = self.on_close.map(std::rc::Rc::new);
        let on_dismiss = self.on_dismiss.map(std::rc::Rc::new);
        let base = div()
            .id(self.id)
            .w(px(320.0))
            .px_3()
            .py_2()
            .flex()
            .items_start()
            .gap_2()
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.border_strong)
            .bg(theme.panel)
            .shadow_lg()
            .child(feedback_icon(tone_icon(self.tone), fg))
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
                            .line_height(px(18.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_xs()
                                .line_height(px(16.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .child({
                let on_close_for_key = on_close.clone();
                let on_dismiss_for_key = on_dismiss.clone();
                IconButton::new(close_id, IconName::X)
                    .size(IconSize::XSmall)
                    .on_click(move |event, window, cx| {
                        if let Some(handler) = &on_close_for_key {
                            handler(event, window, cx);
                        }
                        if let Some(handler) = &on_dismiss_for_key {
                            handler(event, window, cx);
                        }
                    })
            });

        if self.animated {
            base.motion_slide_in(MotionDirection::FromBottom, true)
                .into_any_element()
        } else {
            base.into_any_element()
        }
    }
}
