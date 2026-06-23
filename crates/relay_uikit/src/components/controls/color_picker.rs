use std::{hash::Hash, rc::Rc};

use gpui::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder, px,
};
use relay::{Binding, WindowSignalExt};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ActionHandler, SelectionSource, SharedActionHandler},
    theme::{ActiveTheme, radius, space},
};

/// A selectable color preset for [`ColorPicker`].
#[derive(Clone)]
pub struct ColorPreset<K> {
    key: K,
    label: String,
    color: Hsla,
}

impl<K> ColorPreset<K> {
    pub fn new(key: K, label: impl Into<String>, color: Hsla) -> Self {
        Self {
            key,
            label: label.into(),
            color,
        }
    }

    pub fn key(&self) -> &K {
        &self.key
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
pub struct ColorPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    selected_key: Option<K>,
    presets: Vec<ColorPreset<K>>,
    selection: Option<SelectionSource<K>>,
    on_select: Option<ActionHandler<(K, Hsla)>>,
}

impl<K> ColorPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    pub fn new(id: impl Into<ElementId>, selected_key: K, presets: Vec<ColorPreset<K>>) -> Self {
        Self {
            id: id.into(),
            selected_key: Some(selected_key),
            presets,
            selection: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        binding: Binding<K>,
        presets: Vec<ColorPreset<K>>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key: None,
            presets,
            selection: Some(SelectionSource::binding(binding)),
            on_select: None,
        }
    }

    pub fn selected_key(&self) -> Option<&K> {
        self.selected_key.as_ref()
    }

    pub fn selected_preset(&self) -> Option<&ColorPreset<K>> {
        self.selected_key.as_ref().and_then(|selected_key| {
            self.presets
                .iter()
                .find(|preset| preset.key == *selected_key)
        })
    }

    pub fn on_select(mut self, handler: impl Fn(K, Hsla, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(move |(key, color), window, cx| {
            handler(key, color, window, cx);
        }));
        self
    }
}

impl<K> RenderOnce for ColorPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            selected_key,
            presets,
            selection,
            on_select,
        } = self;
        let selection = selection.or_else(|| {
            selected_key.clone().map(|selected_key| {
                SelectionSource::binding(window.use_binding(
                    (id.clone(), "selected-preset"),
                    cx,
                    || selected_key,
                ))
            })
        });
        let selected_key = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(selected_key);
        let selected = selected_key
            .as_ref()
            .and_then(|selected_key| presets.iter().find(|preset| preset.key == *selected_key))
            .cloned()
            .or_else(|| presets.first().cloned());
        let selected_color = selected
            .as_ref()
            .map_or(theme.accent, |preset| preset.color());
        let selected_label = selected
            .as_ref()
            .map_or("Custom", |preset| preset.label.as_str())
            .to_string();
        let selected_value = color_to_hex(selected_color);
        let select_handler: Option<SharedActionHandler<(K, Hsla)>> = on_select.map(Rc::from);
        let mut grid = div().flex().flex_wrap().gap_2();

        for (index, preset) in presets.into_iter().enumerate() {
            let is_selected = selected_key
                .as_ref()
                .is_some_and(|selected_key| preset.key == *selected_key);
            let preset_selection = selection.clone();
            let preset_handler = select_handler.clone();
            grid = grid.child(preset_button(
                ("color-preset", index),
                preset,
                is_selected,
                preset_selection,
                preset_handler,
            ));
        }

        div()
            .id(id)
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
                                            .text_color(gpui::white())
                                            .child(selected_label),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(gpui::white().opacity(0.82))
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

fn preset_button<K>(
    id: impl Into<ElementId>,
    preset: ColorPreset<K>,
    selected: bool,
    selection: Option<SelectionSource<K>>,
    handler: Option<SharedActionHandler<(K, Hsla)>>,
) -> impl IntoElement
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    let key = preset.key.clone();
    let color = preset.color;
    let interactive = selection.is_some() || handler.is_some();

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
        .when(interactive, |this| {
            this.cursor_pointer().hover(move |style| {
                style
                    .bg(color.opacity(0.12))
                    .border_color(color.opacity(0.72))
            })
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
        .when(interactive, |this| {
            this.on_click(move |_event, window, cx| {
                if let Some(selection) = &selection {
                    selection.select(cx, key.clone());
                }
                if let Some(handler) = &handler {
                    handler((key.clone(), color), window, cx);
                }
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

        assert_eq!(
            picker.selected_preset().map(ColorPreset::key),
            Some(&"blue")
        );
    }
}
