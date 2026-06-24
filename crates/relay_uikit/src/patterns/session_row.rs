use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, Selector};

use crate::{
    display::StatusDot,
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, SelectionBinding},
    theme::{ActiveTheme, BORDER_WIDTH, radius},
    tone::Tone,
};

/// One row in the terminal/session history panel.
#[derive(IntoElement)]
pub struct SessionRow {
    id: ElementId,
    title: String,
    subtitle: String,
    status: Tone,
    active: bool,
    binding: Option<Binding<bool>>,
    selection: Option<SelectionBinding>,
    on_click: Option<ClickHandler>,
}

impl SessionRow {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: subtitle.into(),
            status: Tone::Muted,
            active: false,
            binding: None,
            selection: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        binding: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: subtitle.into(),
            status: Tone::Muted,
            active: false,
            binding: Some(binding),
            selection: None,
            on_click: None,
        }
    }

    pub fn status(mut self, status: Tone) -> Self {
        self.status = status;
        self
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

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SessionRow {
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
            .min_h(px(48.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if active {
                theme.accent_border
            } else {
                gpui::transparent_black()
            })
            .bg(if active {
                theme.accent_bg
            } else {
                gpui::transparent_black()
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
            .child(
                Icon::new(IconName::Terminal)
                    .size(IconSize::Small)
                    .color(self.status.fg(&theme)),
            )
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
                            .child(self.title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(self.subtitle),
                    ),
            )
            .child(StatusDot::new(self.status))
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

    fn row_selection(row: &SessionRow) -> &SelectionBinding {
        let Some(selection) = row.selection.as_ref() else {
            panic!("row should store selection");
        };
        selection
    }

    #[test]
    fn session_row_active_with_selection_model_selects_row_key() {
        let mut app = TestApp::new();
        let (selection, row) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("open"));
            let row = SessionRow::new("session", "codex", "relay/patterns").active_with(
                SelectionBinding::selection_model(selection.clone(), "close"),
            );
            (selection, row)
        });

        app.update(|cx| {
            let selection_binding = row_selection(&row);
            assert!(!selection_binding.is_selected(cx));

            selection_binding.select(cx);

            assert!(selection_binding.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("close"));
        });
    }
}
