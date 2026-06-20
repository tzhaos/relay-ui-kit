use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, Theme, radius, DISABLED_OPACITY, BORDER_WIDTH},
};

/// Standard actions for editing a keybinding row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindingActionKind {
    Edit,
    Reset,
    Clear,
}

impl KeybindingActionKind {
    pub fn icon(self) -> IconName {
        match self {
            Self::Edit => IconName::Pencil,
            Self::Reset => IconName::RotateCcw,
            Self::Clear => IconName::Trash2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Edit => "Edit",
            Self::Reset => "Reset",
            Self::Clear => "Clear",
        }
    }

    fn danger(self) -> bool {
        matches!(self, Self::Clear)
    }
}

struct KeybindingAction {
    kind: KeybindingActionKind,
    handler: Option<ClickHandler>,
    disabled: bool,
}

impl KeybindingAction {
    fn new(kind: KeybindingActionKind) -> Self {
        Self {
            kind,
            handler: None,
            disabled: true,
        }
    }

    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self.disabled = false;
        self
    }
}

/// A compact edit/reset/clear action strip for [`super::KeybindingRow`].
#[derive(IntoElement)]
pub struct KeybindingActions {
    id: ElementId,
    actions: Vec<KeybindingAction>,
}

impl KeybindingActions {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: vec![
                KeybindingAction::new(KeybindingActionKind::Edit),
                KeybindingAction::new(KeybindingActionKind::Reset),
                KeybindingAction::new(KeybindingActionKind::Clear),
            ],
        }
    }

    pub fn on_edit(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.set_handler(KeybindingActionKind::Edit, handler);
        self
    }

    pub fn on_reset(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.set_handler(KeybindingActionKind::Reset, handler);
        self
    }

    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.set_handler(KeybindingActionKind::Clear, handler);
        self
    }

    pub fn enabled_actions(&self) -> usize {
        self.actions
            .iter()
            .filter(|action| !action.disabled)
            .count()
    }

    fn set_handler(
        &mut self,
        kind: KeybindingActionKind,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) {
        if let Some(action) = self.actions.iter_mut().find(|action| action.kind == kind) {
            *action = KeybindingAction::new(kind).on_click(handler);
        }
    }
}

impl RenderOnce for KeybindingActions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut strip = div()
            .id(self.id)
            .h(px(26.0))
            .flex()
            .items_center()
            .gap(px(BORDER_WIDTH))
            .rounded(px(radius::MD))
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel_alt)
            .overflow_hidden();

        for (index, action) in self.actions.into_iter().enumerate() {
            strip = strip.child(action_button(action, index, theme));
        }

        strip
    }
}

fn action_button(action: KeybindingAction, index: usize, theme: Theme) -> impl IntoElement {
    let kind = action.kind;
    let icon_color = if kind.danger() {
        theme.danger
    } else {
        theme.text_muted
    };

    div()
        .id(("keybinding-action", index))
        .size(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.0))
        .font_weight(FontWeight::MEDIUM)
        .when(action.disabled, |this| this.opacity(DISABLED_OPACITY))
        .when(!action.disabled, |this| {
            this.cursor_pointer().hover(move |style| {
                if kind.danger() {
                    style.bg(theme.danger.opacity(0.12))
                } else {
                    style.bg(theme.hover)
                }
            })
        })
        .child(
            Icon::new(kind.icon())
                .size(IconSize::XSmall)
                .color(icon_color),
        )
        .when_some(
            action.handler.filter(|_| !action.disabled),
            |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keybinding_action_kind_uses_specific_icons() {
        assert_eq!(KeybindingActionKind::Edit.icon(), IconName::Pencil);
    }

    #[test]
    fn keybinding_actions_start_disabled() {
        let actions = KeybindingActions::new("actions");

        assert_eq!(actions.enabled_actions(), 0);
    }
}
