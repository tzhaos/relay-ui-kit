use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, Selector};

use crate::{
    display::StatusDot,
    interaction::{ClickHandler, SelectionBinding},
    theme::{ActiveTheme, radius},
    tone::Tone,
};

/// A terminal tab in a session tab strip.
#[derive(IntoElement)]
pub struct TabStrip {
    id: ElementId,
    label: String,
    active: bool,
    status: Tone,
    binding: Option<Binding<bool>>,
    selection: Option<SelectionBinding>,
    on_click: Option<ClickHandler>,
}

impl TabStrip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            active: false,
            status: Tone::Muted,
            binding: None,
            selection: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        label: impl Into<String>,
        binding: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            active: false,
            status: Tone::Muted,
            binding: Some(binding),
            selection: None,
            on_click: None,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn active_with(mut self, selection: SelectionBinding) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn active_by<K>(self, selector: Selector<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        self.active_with(SelectionBinding::selector(selector, key))
    }

    pub fn status(mut self, status: Tone) -> Self {
        self.status = status;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TabStrip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let selection = self.selection;
        let active = if let Some(selection) = &selection {
            selection.is_selected(cx)
        } else {
            binding.as_ref().map_or(self.active, |b| b.get(cx))
        };
        let handler = self.on_click;
        let clickable = selection.is_some() || binding.is_some() || handler.is_some();

        div()
            .id(self.id)
            .h(px(30.0))
            .max_w(px(190.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if active {
                theme.accent_border
            } else {
                theme.border
            })
            .bg(if active { theme.panel } else { theme.panel_alt })
            .text_color(if active {
                theme.text
            } else {
                theme.text_secondary
            })
            .when(clickable, |this| this.cursor_pointer())
            .when(clickable && !active, |this| {
                this.hover(move |style| style.bg(theme.hover))
            })
            .when(clickable, |this| {
                this.on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
            })
            .child(StatusDot::new(self.status).size(6.0))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label),
            )
            .when(clickable, |this| {
                this.on_click(move |event, window, cx| {
                    if let Some(selection) = &selection {
                        selection.select(cx);
                    } else if let Some(binding) = &binding {
                        binding.update(cx, |active| {
                            *active = !*active;
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{SelectionModel, init};

    use super::*;

    fn tab_selection(tab: &TabStrip) -> &SelectionBinding {
        let Some(selection) = tab.selection.as_ref() else {
            panic!("tab should store selection");
        };
        selection
    }

    #[test]
    fn tab_strip_active_with_selection_model_selects_row_key() {
        let mut app = TestApp::new();
        let (selection, tab) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("terminal"));
            let tab = TabStrip::new("preview-tab", "Preview").active_with(
                SelectionBinding::selection_model(selection.clone(), "preview"),
            );
            (selection, tab)
        });

        app.update(|cx| {
            let selection_binding = tab_selection(&tab);
            assert!(!selection_binding.is_selected(cx));

            selection_binding.select(cx);

            assert!(selection_binding.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("preview"));
        });
    }
}
