use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::SelectHandler,
    theme::ActiveTheme,
};

/// One tab in a [`Tabs`] bar.
pub struct Tab {
    key: &'static str,
    label: &'static str,
    icon: Option<IconName>,
    count: Option<usize>,
}

impl Tab {
    pub fn new(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            icon: None,
            count: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

/// An underline tab bar.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    tabs: Vec<Tab>,
    active: &'static str,
    binding: Option<Binding<&'static str>>,
    on_select: Option<SelectHandler>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>, tabs: Vec<Tab>) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: "",
            binding: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        tabs: Vec<Tab>,
        binding: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: "",
            binding: Some(binding),
            on_select: None,
        }
    }

    pub fn active(mut self, active: &'static str) -> Self {
        self.active = active;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let active = binding.as_ref().map_or(self.active, |b| b.get(cx));
        let handler = self.on_select.map(std::rc::Rc::new);
        let mut row = div()
            .id(self.id)
            .h(px(36.0))
            .w_full()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(theme.border);

        for (index, tab) in self.tabs.into_iter().enumerate() {
            let is_active = tab.key == active;
            let key = tab.key;
            let handler = handler.clone();
            let binding = binding.clone();
            let (fg, underline) = if is_active {
                (theme.text, theme.accent)
            } else {
                (theme.text_muted, gpui::transparent_black())
            };
            let clickable = !is_active && (binding.is_some() || handler.is_some());

            let cell = div()
                .id(("tab", index))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(fg)
                .border_b_2()
                .border_color(underline)
                .when(!is_active, |this| {
                    this.cursor_pointer()
                        .hover(move |s| s.text_color(theme.text_secondary))
                })
                .when_some(tab.icon, |this, icon| {
                    let c = if is_active {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    this.child(Icon::new(icon).size(IconSize::Small).color(c))
                })
                .child(tab.label)
                .when_some(tab.count, |this, count| {
                    this.child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(format!("({count})")),
                    )
                })
                .when(clickable, |this| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        if let Some(binding) = &binding {
                            binding.set(cx, key);
                        }
                        if let Some(handler) = &handler {
                            handler(key, window, cx);
                        }
                        cx.stop_propagation();
                    })
                });
            row = row.child(cell);
        }
        row
    }
}
