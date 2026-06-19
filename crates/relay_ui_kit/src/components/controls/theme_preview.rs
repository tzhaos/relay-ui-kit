use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, linear_color_stop,
    linear_gradient, prelude::FluentBuilder, px, rgb,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, Theme, radius},
};

/// Appearance thumbnail variants used by [`ThemePreviewCard`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreviewKind {
    System,
    Light,
    Dark,
}

impl ThemePreviewKind {
    pub fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

/// A compact selectable appearance preview.
#[derive(IntoElement)]
pub struct ThemePreviewCard {
    id: ElementId,
    kind: ThemePreviewKind,
    label: String,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl ThemePreviewCard {
    pub fn new(id: impl Into<ElementId>, kind: ThemePreviewKind) -> Self {
        Self {
            id: id.into(),
            kind,
            label: kind.label().into(),
            selected: false,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn key(&self) -> &'static str {
        self.kind.key()
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ThemePreviewCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let border = if self.selected {
            theme.accent
        } else {
            theme.border
        };
        let id = self.id;
        let handler = self.on_click;

        div()
            .id(id.clone())
            .w(px(156.0))
            .p_1()
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(radius::LG))
            .border_1()
            .border_color(border)
            .bg(if self.selected {
                theme.accent_bg
            } else {
                theme.panel
            })
            .cursor_pointer()
            .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
            .child(preview_frame((id.clone(), "preview"), self.kind, theme))
            .child(
                div()
                    .px_1()
                    .pb_1()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.label),
                    )
                    .when(self.selected, |this| {
                        this.child(
                            div()
                                .size(px(16.0))
                                .rounded_full()
                                .bg(theme.accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Icon::new(IconName::Check)
                                        .size(IconSize::XSmall)
                                        .color(theme.on_accent),
                                ),
                        )
                    }),
            )
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

#[derive(Clone, Copy)]
struct PreviewPalette {
    app_bg: Hsla,
    chrome: Hsla,
    panel: Hsla,
    text: Hsla,
    accent: Hsla,
}

impl PreviewPalette {
    fn light(theme: Theme) -> Self {
        Self {
            app_bg: theme.app_bg,
            chrome: theme.chrome,
            panel: theme.panel,
            text: theme.text,
            accent: theme.accent,
        }
    }

    fn dark() -> Self {
        Self {
            app_bg: rgb(0x181b20).into(),
            chrome: rgb(0x20242b).into(),
            panel: rgb(0x2a2f37).into(),
            text: rgb(0xd8dbe0).into(),
            accent: rgb(0x22c55e).into(),
        }
    }
}

fn preview_frame(id: impl Into<ElementId>, kind: ThemePreviewKind, theme: Theme) -> AnyElement {
    let light = PreviewPalette::light(theme);
    let dark = PreviewPalette::dark();

    match kind {
        ThemePreviewKind::System => div()
            .id(id)
            .h(px(86.0))
            .flex()
            .overflow_hidden()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(theme.border)
            .child(preview_half("theme-preview-system-light-half", light))
            .child(preview_half("theme-preview-system-dark-half", dark))
            .into_any_element(),
        ThemePreviewKind::Light => preview_scene(id, light, theme.border).into_any_element(),
        ThemePreviewKind::Dark => preview_scene(id, dark, theme.border).into_any_element(),
    }
}

fn preview_half(id: impl Into<ElementId>, palette: PreviewPalette) -> impl IntoElement {
    div()
        .flex_1()
        .child(preview_scene(id, palette, palette.chrome))
}

fn preview_scene(
    id: impl Into<ElementId>,
    palette: PreviewPalette,
    border: Hsla,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(86.0))
        .overflow_hidden()
        .rounded(px(radius::MD))
        .border_1()
        .border_color(border)
        .bg(linear_gradient(
            135.0,
            linear_color_stop(palette.panel, 0.0),
            linear_color_stop(palette.app_bg, 1.0),
        ))
        .child(
            div()
                .h(px(18.0))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .bg(palette.chrome)
                .child(dot(palette.accent))
                .child(dot(palette.text.opacity(0.24)))
                .child(dot(palette.text.opacity(0.16))),
        )
        .child(
            div()
                .p_2()
                .flex()
                .gap_2()
                .child(
                    div()
                        .w(px(34.0))
                        .h(px(48.0))
                        .rounded(px(radius::SM))
                        .bg(palette.chrome),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(line(palette.text.opacity(0.72), 48.0))
                        .child(line(palette.text.opacity(0.28), 64.0))
                        .child(line(palette.accent, 38.0)),
                ),
        )
}

fn dot(color: Hsla) -> impl IntoElement {
    div().size(px(5.0)).rounded_full().bg(color)
}

fn line(color: Hsla, width: f32) -> impl IntoElement {
    div().h(px(5.0)).w(px(width)).rounded_full().bg(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_preview_kind_keys_match_settings_values() {
        assert_eq!(ThemePreviewKind::System.key(), "system");
        assert_eq!(ThemePreviewKind::Light.key(), "light");
        assert_eq!(ThemePreviewKind::Dark.key(), "dark");
    }

    #[test]
    fn theme_preview_card_exposes_kind_key() {
        let card = ThemePreviewCard::new("theme-card", ThemePreviewKind::Dark);

        assert_eq!(card.key(), "dark");
    }
}
