use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};

use relay_ui_core::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, radius, space},
};

/// A compact elevated panel for inline details and small action groups.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    title: Option<String>,
    icon: Option<IconName>,
    width: f32,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            icon: None,
            width: 280.0,
            children: Vec::new(),
            footer: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .id(self.id)
            .w(px(self.width))
            .p(px(space::SM))
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude()
            .when(self.title.is_some() || self.icon.is_some(), |this| {
                this.child(
                    div()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when_some(self.icon, |this, icon| {
                            this.child(
                                Icon::new(icon)
                                    .size(IconSize::Small)
                                    .color(theme.text_muted),
                            )
                        })
                        .when_some(self.title, |this, title| {
                            this.child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(title),
                            )
                        }),
                )
            })
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.children),
            )
            .when_some(self.footer, |this, footer| {
                this.child(
                    div()
                        .mt_1()
                        .pt_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(footer),
                )
            })
            .motion_slide_in(MotionDirection::FromTop, true)
    }
}
