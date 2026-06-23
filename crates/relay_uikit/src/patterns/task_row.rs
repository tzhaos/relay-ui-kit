use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder, px,
};
use relay::{Binding, Selector};

use crate::{
    StatusDot,
    interaction::{ClickHandler, SelectionBinding},
    list::{ListItem, ListItemSpacing},
    theme::{ActiveTheme, space},
    tone::Tone,
};

/// Metadata for one [`TaskRow`].
pub struct TaskRowData {
    pub title: String,
    pub status_label: String,
    pub status_tone: Tone,
    pub branch: Option<String>,
    pub changed: usize,
    pub review: usize,
}

/// A fixed-height task row for the left rail.
#[derive(IntoElement)]
pub struct TaskRow {
    id: ElementId,
    data: TaskRowData,
    selected: bool,
    binding: Option<Binding<bool>>,
    selection: Option<SelectionBinding>,
    on_click: Option<ClickHandler>,
}

impl TaskRow {
    pub fn new(id: impl Into<ElementId>, data: TaskRowData) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            binding: None,
            selection: None,
            on_click: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, data: TaskRowData, binding: Binding<bool>) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            binding: Some(binding),
            selection: None,
            on_click: None,
        }
    }

    pub fn selected_with(mut self, selection: SelectionBinding) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn selected_by<K>(self, selector: Selector<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        self.selected_with(SelectionBinding::selector(selector, key))
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

impl RenderOnce for TaskRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let data = self.data;
        let meta = task_meta(&data);
        let binding = self.binding;
        let selection = self.selection;
        let selected = if let Some(selection) = &selection {
            selection.is_selected(cx)
        } else {
            binding.as_ref().map_or(self.selected, |b| b.get(cx))
        };
        let handler = self.on_click;

        let mut row = ListItem::new(self.id)
            .spacing(ListItemSpacing::Relaxed)
            .height(px(space::TASK_ROW))
            .selected(selected)
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(StatusDot::new(data.status_tone))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(data.title),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(data.status_tone.fg(&theme))
                                    .child(data.status_label),
                            ),
                    )
                    .when(!meta.is_empty(), |this| {
                        this.child(
                            div()
                                .pl(px(15.0))
                                .truncate()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(meta),
                        )
                    }),
            );

        let has_click = selection.is_some() || binding.is_some() || handler.is_some();
        if has_click {
            row = row.on_click(move |event, window, cx| {
                if let Some(selection) = &selection {
                    selection.select(cx);
                } else if let Some(binding) = &binding {
                    binding.update(cx, |selected| {
                        *selected = !*selected;
                        true
                    });
                }
                if let Some(handler) = &handler {
                    handler(event, window, cx);
                }
            });
        }

        row
    }
}

fn task_meta(data: &TaskRowData) -> String {
    let mut parts = Vec::new();
    if let Some(branch) = &data.branch {
        parts.push(branch.clone());
    }
    if data.changed > 0 {
        parts.push(format!("{}±", data.changed));
    }
    if data.review > 0 {
        parts.push(format!("{} review", data.review));
    }
    parts.join("  ·  ")
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{SelectionModel, init};

    use super::*;

    fn row_data() -> TaskRowData {
        TaskRowData {
            title: "Ship v2".to_string(),
            status_label: "ready".to_string(),
            status_tone: Tone::Accent,
            branch: Some("feat/v2".to_string()),
            changed: 3,
            review: 1,
        }
    }

    #[test]
    fn task_row_selected_with_selection_model_selects_row_key() {
        let mut app = TestApp::new();
        let (selection, row) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("open"));
            let row = TaskRow::new("task", row_data()).selected_with(
                SelectionBinding::selection_model(selection.clone(), "close"),
            );
            (selection, row)
        });

        app.update(|cx| {
            let selection_binding = row.selection.as_ref().expect("row should store selection");
            assert!(!selection_binding.is_selected(cx));

            selection_binding.select(cx);

            assert!(selection_binding.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("close"));
        });
    }
}
