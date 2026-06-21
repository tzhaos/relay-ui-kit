use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::SelectHandler,
    theme::{ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY, radius},
};

use super::KeybindingShortcut;

/// One command row inside a command palette or launcher.
#[derive(IntoElement)]
pub struct CommandRow {
    id: ElementId,
    key: &'static str,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
    shortcut: Option<KeybindingShortcut>,
    selected: bool,
    disabled: bool,
    binding: Option<Binding<&'static str>>,
    on_select: Option<SelectHandler>,
}

impl CommandRow {
    pub fn new(id: impl Into<ElementId>, key: &'static str, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            binding: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        key: &'static str,
        label: impl Into<String>,
        binding: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            binding: Some(binding),
            on_select: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: KeybindingShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for CommandRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let selected = binding.as_ref().map_or(self.selected, |b| b.get(cx) == self.key);
        let handler = self.on_select;
        let disabled = self.disabled || (handler.is_none() && binding.is_none());
        let key = self.key;

        div()
            .id(self.id)
            .min_h(px(44.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .when(selected, |this| this.bg(theme.selection))
            .when(disabled, |this| this.opacity(DISABLED_OPACITY))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    div()
                        .size(px(24.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(icon)
                                .size(IconSize::Small)
                                .color(theme.text_secondary),
                        ),
                )
            })
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
                            .child(self.label),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .when_some(self.shortcut, |this, shortcut| this.child(shortcut))
            .when(!disabled, |this| {
                this.on_click(move |_: &ClickEvent, window, cx| {
                    if let Some(binding) = &binding {
                        binding.set(cx, key);
                    }
                    if let Some(handler) = &handler {
                        handler(key, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
    }
}
