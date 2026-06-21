use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    list::ListItem,
    theme::{ActiveTheme, radius, space},
};

/// A top-level navigation row with a leading icon and optional count.
#[derive(IntoElement)]
pub struct NavRow {
    id: ElementId,
    icon: IconName,
    label: String,
    count: Option<usize>,
    selected: bool,
    binding: Option<Binding<bool>>,
    on_click: Option<ClickHandler>,
}

impl NavRow {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            count: None,
            selected: false,
            binding: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        icon: IconName,
        label: impl Into<String>,
        binding: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            count: None,
            selected: false,
            binding: Some(binding),
            on_click: None,
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

impl RenderOnce for NavRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let selected = binding.as_ref().map_or(self.selected, |b| b.get(cx));
        let (fg, icon_color) = if selected {
            (theme.text, theme.accent)
        } else {
            (theme.text_secondary, theme.text_muted)
        };
        let handler = self.on_click;

        let mut row = ListItem::new(self.id)
            .height(px(space::ROW_MD))
            .selected(selected)
            .start_slot(Icon::new(self.icon).size(IconSize::Small).color(icon_color))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(fg)
                    .child(self.label),
            );

        if let Some(count) = self.count {
            row = row.end_slot(
                div()
                    .min_w(px(18.0))
                    .h(px(18.0))
                    .px_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(radius::SM))
                    .bg(theme.panel_alt)
                    .text_color(theme.text_muted)
                    .text_size(px(11.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(count.to_string()),
            );
        }

        let has_click = binding.is_some() || handler.is_some();
        if has_click {
            row = row.on_click(move |event, window, cx| {
                if let Some(binding) = &binding {
                    binding.update(cx, |selected| {
                        *selected = !*selected;
                        true
                    });
                }
                if let Some(handler) = &handler {
                    handler(event, window, cx);
                }
            });
        }

        row
    }
}
