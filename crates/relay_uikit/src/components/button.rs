//! Button components: a labelled [`Button`] and a square [`IconButton`].
//!
//! Both are `RenderOnce` builders with a generic click callback
//! (`Fn(&ClickEvent, &mut Window, &mut App)`), so they carry no dependency on any
//! concrete view — the gallery and the real workbench wire the same component to
//! different handlers.

use gpui::{App, ClickEvent, ElementId, FocusHandle, FontWeight, IntoElement, RenderOnce, Window};

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
    focus_handle: Option<FocusHandle>,
    aria_expanded: Option<bool>,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// Create a button with a stable id and visible text label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Secondary,
            icon: None,
            disabled: false,
            focus_handle: None,
            aria_expanded: None,
            on_click: None,
        }
    }

    /// Set the visual emphasis for this action.
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

    /// Add a leading icon while keeping the text label as the accessible name.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Disable pointer and keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Track focus on the rendered button with a host-owned focus handle.
    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    /// Mark the button as the trigger for an expandable surface.
    pub fn aria_expanded(mut self, expanded: bool) -> Self {
        self.aria_expanded = Some(expanded);
        self
    }

    /// Run the action for both pointer clicks and keyboard activation.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let (bg, border, fg, hover_bg, hover_border, hover_fg) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent,
                theme.accent,
                theme.on_accent,
                theme.accent.opacity(0.85),
                theme.accent,
                theme.on_accent,
            ),
            ButtonVariant::Danger => (
                theme.danger,
                theme.danger,
                gpui::white(),
                theme.danger.opacity(0.85),
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
        .aria_label(self.label.clone())
        .disabled(self.disabled)
        .on_click(self.on_click);

        if let Some(expanded) = self.aria_expanded {
            button = button.aria_expanded(expanded);
        }

        if let Some(focus_handle) = self.focus_handle {
            button = button.track_focus(focus_handle);
        }

        if let Some(icon) = self.icon {
            button = button.child(Icon::new(icon).size(IconSize::Small).color(icon_color));
        }

        button.child(self.label)
    }
}

/// A square, borderless icon button — toolbar actions, row affordances.
///
/// Prefer [`Button`] when a textual label fits naturally in the layout. Use
/// [`IconButton`] when the surrounding chrome already establishes the action's
/// context and the compact footprint matters.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    size: IconSize,
    active: bool,
    disabled: bool,
    aria_label: Option<String>,
    focus_handle: Option<FocusHandle>,
    aria_expanded: Option<bool>,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    /// Create an icon button with a stable id and icon glyph.
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            size: IconSize::Small,
            active: false,
            disabled: false,
            aria_label: None,
            focus_handle: None,
            aria_expanded: None,
            on_click: None,
        }
    }

    /// Override the glyph size while keeping the same button chrome.
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

    /// Override the accessible label.
    ///
    /// This is recommended whenever the same icon can mean different things in
    /// different product surfaces.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Track focus on the rendered button with a host-owned focus handle.
    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    /// Mark the button as the trigger for an expandable surface.
    pub fn aria_expanded(mut self, expanded: bool) -> Self {
        self.aria_expanded = Some(expanded);
        self
    }

    /// Run the action for both pointer clicks and keyboard activation.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

fn resolved_icon_button_label(icon: IconName, aria_label: Option<String>) -> String {
    aria_label.unwrap_or_else(|| icon.label().to_string())
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.active {
            theme.accent
        } else {
            theme.text_muted
        };
        let aria_label = resolved_icon_button_label(self.icon, self.aria_label);

        let mut button = ButtonLike::new(
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
        .toggled(self.active)
        .disabled(self.disabled)
        .on_click(self.on_click);

        if let Some(expanded) = self.aria_expanded {
            button = button.aria_expanded(expanded);
        }

        if let Some(focus_handle) = self.focus_handle {
            button = button.track_focus(focus_handle);
        }

        button = button.aria_label(aria_label);

        button.child(Icon::new(self.icon).size(self.size).color(fg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_aria_expanded_builder_stores_value() {
        let button = Button::new("button", "Open").aria_expanded(true);

        assert_eq!(button.aria_expanded, Some(true));
    }

    #[test]
    fn icon_button_aria_expanded_builder_stores_value() {
        let button = IconButton::new("button", IconName::ChevronDown).aria_expanded(false);

        assert_eq!(button.aria_expanded, Some(false));
    }

    #[test]
    fn icon_button_uses_icon_name_as_default_aria_label() {
        assert_eq!(
            resolved_icon_button_label(IconName::Ellipsis, None),
            "More actions"
        );
    }

    #[test]
    fn icon_button_prefers_explicit_aria_label() {
        assert_eq!(
            resolved_icon_button_label(IconName::Ellipsis, Some("Open terminal menu".into())),
            "Open terminal menu"
        );
    }
}
