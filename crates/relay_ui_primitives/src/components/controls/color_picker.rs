use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ColorSelectHandler, SharedColorSelectHandler},
    theme::{ActiveTheme, radius, space},
};

/// A selectable color preset for [`ColorPicker`].
#[derive(Clone)]
pub struct ColorPreset {
    key: &'static str,
    label: String,
    color: Hsla,
}

impl ColorPreset {
    pub fn new(key: &'static str, label: impl Into<String>, color: Hsla) -> Self {
        Self {
            key,
            label: label.into(),
            color,
        }
    }

    pub fn key(&self) -> &'static str {
        self.key
    }

    pub fn color(&self) -> Hsla {
        self.color
    }

    pub fn hex_value(&self) -> String {
        color_to_hex(self.color)
    }
}

/// A compact preset-based color picker.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    selected_key: &'static str,
    presets: Vec<ColorPreset>,
    on_select: Option<ColorSelectHandler>,
}

impl ColorPicker {
    pub fn new(
        id: impl Into<ElementId>,
        selected_key: &'static str,
        presets: Vec<ColorPreset>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key,
            presets,
            on_select: None,
        }
    }

    pub fn selected_key(&self) -> &'static str {
        self.selected_key
    }

    pub fn selected_preset(&self) -> Option<&ColorPreset> {
        self.presets
            .iter()
            .find(|preset| preset.key == self.selected_key)
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, Hsla, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ColorPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selected = self
            .selected_preset()
            .cloned()
            .or_else(|| self.presets.first().cloned());
        let selected_color = selected
            .as_ref()
            .map_or(theme.accent, |preset| preset.color());
        let selected_label = selected
            .as_ref()
            .map_or("Custom", |preset| preset.label.as_str())
            .to_string();
        let selected_value = color_to_hex(selected_color);
        let select_handler: Option<SharedColorSelectHandler> = self.on_select.map(Rc::from);
        let mut grid = div().flex().flex_wrap().gap_2();

        for preset in self.presets {
            let is_selected = preset.key == self.selected_key;
            grid = grid.child(preset_button(
                (self.id.clone(), preset.key),
                preset,
                is_selected,
                select_handler.clone(),
            ));
        }

        div()
            .id(self.id)
            .w(px(320.0))
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .h(px(64.0))
                    .rounded(px(radius::MD))
                    .overflow_hidden()
                    .border_1()
                    .border_color(theme.border)
                    .bg(linear_gradient(
                        135.0,
                        linear_color_stop(selected_color, 0.0),
                        linear_color_stop(theme.panel_alt, 1.0),
                    ))
                    .child(
                        div()
                            .size_full()
                            .p_2()
                            .flex()
                            .items_end()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(space::XXS))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.text)
                                            .child(selected_label),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.text_muted)
                                            .child(selected_value),
                                    ),
                            )
                            .child(
                                div()
                                    .size(px(24.0))
                                    .rounded(px(radius::MD))
                                    .bg(selected_color)
                                    .border_1()
                                    .border_color(gpui::white().opacity(0.82)),
                            ),
                    ),
            )
            .child(hue_row())
            .child(grid)
    }
}

fn preset_button(
    id: impl Into<ElementId>,
    preset: ColorPreset,
    selected: bool,
    handler: Option<SharedColorSelectHandler>,
) -> impl IntoElement {
    let key = preset.key;
    let color = preset.color;

    div()
        .id(id)
        .h(px(30.0))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(radius::MD))
        .border_1()
        .border_color(if selected {
            color
        } else {
            gpui::black().opacity(0.08)
        })
        .bg(if selected {
            color.opacity(0.16)
        } else {
            gpui::transparent_black()
        })
        .cursor_pointer()
        .hover(move |style| {
            style
                .bg(color.opacity(0.12))
                .border_color(color.opacity(0.72))
        })
        .child(
            div()
                .size(px(16.0))
                .rounded(px(radius::SM))
                .bg(color)
                .border_1()
                .border_color(gpui::black().opacity(0.08)),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .child(preset.label),
        )
        .when(selected, |this| {
            this.child(
                Icon::new(IconName::Check)
                    .size(IconSize::XSmall)
                    .color(color),
            )
        })
        .when_some(handler, |this, handler| {
            this.on_click(move |_event, window, cx| {
                handler(key, color, window, cx);
                cx.stop_propagation();
            })
        })
}

fn hue_row() -> impl IntoElement {
    div()
        .h(px(8.0))
        .flex()
        .overflow_hidden()
        .rounded_full()
        .child(hue_segment(0.0))
        .child(hue_segment(0.08))
        .child(hue_segment(0.16))
        .child(hue_segment(0.32))
        .child(hue_segment(0.55))
        .child(hue_segment(0.7))
        .child(hue_segment(0.86))
}

fn hue_segment(hue: f32) -> impl IntoElement {
    div().flex_1().bg(hsla(hue, 0.72, 0.52, 1.0))
}

fn color_to_hex(color: Hsla) -> String {
    let rgba = color.to_rgb();
    let r = (rgba.r * 255.0).round() as u8;
    let g = (rgba.g * 255.0).round() as u8;
    let b = (rgba.b * 255.0).round() as u8;

    format!("#{r:02X}{g:02X}{b:02X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_preset_formats_hex_value() {
        let preset = ColorPreset::new("green", "Green", gpui::rgb(0x16a34a).into());

        assert_eq!(preset.hex_value(), "#16A34A");
    }

    #[test]
    fn color_picker_finds_selected_preset() {
        let picker = ColorPicker::new(
            "picker",
            "blue",
            vec![
                ColorPreset::new("green", "Green", gpui::rgb(0x16a34a).into()),
                ColorPreset::new("blue", "Blue", gpui::rgb(0x2563eb).into()),
            ],
        );

        assert_eq!(picker.selected_preset().map(ColorPreset::key), Some("blue"));
    }
}
