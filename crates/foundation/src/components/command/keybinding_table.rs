use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    command::KeybindingShortcut,
    theme::{ActiveTheme, radius, space},
};

/// One command row in a keybinding table.
pub struct KeybindingRow {
    command: String,
    description: Option<String>,
    shortcut: Option<KeybindingShortcut>,
    action: Option<AnyElement>,
}

impl KeybindingRow {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            description: None,
            shortcut: None,
            action: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn shortcut(mut self, shortcut: KeybindingShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

/// A compact table for keybinding management screens.
#[derive(IntoElement)]
pub struct KeybindingTable {
    rows: Vec<KeybindingRow>,
}

impl KeybindingTable {
    pub fn new(rows: Vec<KeybindingRow>) -> Self {
        Self { rows }
    }
}

impl RenderOnce for KeybindingTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let last_index = self.rows.len().saturating_sub(1);
        let mut table = div()
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden();

        table = table.child(
            div()
                .h(px(34.0))
                .px_3()
                .flex()
                .items_center()
                .gap_4()
                .border_b_1()
                .border_color(theme.border)
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(div().flex_1().child("COMMAND"))
                .child(div().w(px(240.0)).child("BINDING")),
        );

        for (index, row) in self.rows.into_iter().enumerate() {
            table = table.child(render_row(row, index == last_index, theme));
        }

        table
    }
}

fn render_row(row: KeybindingRow, last: bool, theme: crate::Theme) -> impl IntoElement {
    div()
        .min_h(px(56.0))
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .gap_4()
        .when(!last, |this| this.border_b_1().border_color(theme.border))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(space::XXS))
                .child(
                    div()
                        .truncate()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child(row.command),
                )
                .when_some(row.description, |this, description| {
                    this.child(
                        div()
                            .truncate()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(description),
                    )
                }),
        )
        .child(
            div()
                .w(px(240.0))
                .min_w_0()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .when_some(row.shortcut, |this, shortcut| this.child(shortcut))
                .when_some(row.action, |this, action| this.child(action)),
        )
}
