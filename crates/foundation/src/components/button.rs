//! Button components: a labelled [`Button`] and a square [`IconButton`].
//!
//! Both are `RenderOnce` builders with a generic click callback
//! (`Fn(&ClickEvent, &mut Window, &mut App)`), so they carry no dependency on any
//! concrete view — the gallery and the real workbench wire the same component to
//! different handlers.

use gpui::{App, ClickEvent, ElementId, FontWeight, IntoElement, RenderOnce, Window};

use crate::{
    components::button_like::{ButtonLike, ButtonLikeColors},
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::ActiveTheme,
};

/// Visual emphasis for a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled accent — the single primary action in a surface.
    Primary,
    /// Filled destructive action.
    Danger,
    /// Outlined neutral — standard actionable controls.
    Secondary,
    /// Borderless text — low-stakes secondary actions.
    Ghost,
}

/// A compact labelled button with an optional leading icon.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: String,
    variant: ButtonVariant,
    icon: Option<IconName>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Secondary,
            icon: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shorthand for the single primary action in a surface.
    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    /// Shorthand for a borderless, low-stakes action.
    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    crate::callback_builder!(on_click, on_click, ClickEvent);
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let (bg, border, fg, hover_bg, hover_border, hover_fg) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent,
                theme.accent,
                theme.on_accent,
                theme.accent,
                theme.accent,
                theme.on_accent,
            ),
            ButtonVariant::Danger => (
                theme.danger,
                theme.danger,
                gpui::white(),
                theme.danger,
                theme.danger,
                gpui::white(),
            ),
            ButtonVariant::Secondary => (
                theme.panel,
                theme.border_strong,
                theme.text,
                theme.hover,
                theme.border_strong,
                theme.text,
            ),
            ButtonVariant::Ghost => (
                gpui::transparent_black(),
                gpui::transparent_black(),
                theme.text_secondary,
                theme.hover,
                gpui::transparent_black(),
                theme.text,
            ),
        };

        let icon_color = fg;
        let mut button = ButtonLike::new(
            self.id,
            ButtonLikeColors::new(bg, border, fg, hover_bg, hover_border, hover_fg),
        )
        .height(28.0)
        .padding_x(8.0)
        .gap(4.0)
        .text_size(12.0)
        .font_weight(FontWeight::MEDIUM)
        .disabled(self.disabled)
        .on_click(self.on_click);

        if let Some(icon) = self.icon {
            button = button.child(Icon::new(icon).size(IconSize::Small).color(icon_color));
        }

        button.child(self.label)
    }
}

/// A square, borderless icon button — toolbar actions, row affordances.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    size: IconSize,
    active: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            size: IconSize::Small,
            active: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Render in the active/selected state (accent foreground).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    crate::callback_builder!(on_click, on_click, ClickEvent);
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.active {
            theme.accent
        } else {
            theme.text_muted
        };

        ButtonLike::new(
            self.id,
            ButtonLikeColors::new(
                gpui::transparent_black(),
                gpui::transparent_black(),
                fg,
                theme.hover,
                gpui::transparent_black(),
                theme.text,
            ),
        )
        .size(26.0)
        .disabled(self.disabled)
        .on_click(self.on_click)
        .child(Icon::new(self.icon).size(self.size).color(fg))
    }
}
