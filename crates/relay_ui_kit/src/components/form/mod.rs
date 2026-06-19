use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, radius};

/// Label text for a form field.
#[derive(IntoElement)]
pub struct FieldLabel {
    text: String,
}

impl FieldLabel {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for FieldLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text)
            .child(self.text)
    }
}

/// Muted supporting text for a form field.
#[derive(IntoElement)]
pub struct FieldDescription {
    text: String,
}

impl FieldDescription {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for FieldDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .text_xs()
            .text_color(cx.theme().text_muted)
            .child(self.text)
    }
}

/// A settings row with label/description on the left and a control on the right.
#[derive(IntoElement)]
pub struct SettingsRow {
    label: String,
    description: Option<String>,
    control: Option<AnyElement>,
    last: bool,
}

impl SettingsRow {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            control: None,
            last: false,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn control(mut self, control: impl IntoElement) -> Self {
        self.control = Some(control.into_any_element());
        self
    }

    pub fn last(mut self, last: bool) -> Self {
        self.last = last;
        self
    }
}

impl RenderOnce for SettingsRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .min_h(px(50.0))
            .px_3()
            .py_2()
            .flex()
            .items_center()
            .gap_4()
            .when(!self.last, |this| {
                this.border_b_1().border_color(theme.border)
            })
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(FieldLabel::new(self.label))
                    .when_some(self.description, |this, description| {
                        this.child(FieldDescription::new(description))
                    }),
            )
            .when_some(self.control, |this, control| {
                this.child(div().flex_shrink_0().child(control))
            })
    }
}

/// A grouped settings panel.
#[derive(IntoElement)]
pub struct SettingsSection {
    title: String,
    rows: Vec<SettingsRow>,
    actions: Option<AnyElement>,
}

impl SettingsSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            actions: None,
        }
    }

    pub fn row(mut self, row: SettingsRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

impl RenderOnce for SettingsSection {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let last_index = self.rows.len().saturating_sub(1);
        let mut body = div()
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden();

        for (index, row) in self.rows.drain(..).enumerate() {
            body = body.child(row.last(index == last_index));
        }

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_muted)
                            .child(self.title.to_uppercase()),
                    )
                    .when_some(self.actions, |this, actions| this.child(actions)),
            )
            .child(body)
    }
}
