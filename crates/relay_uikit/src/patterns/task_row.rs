use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    StatusDot,
    interaction::ClickHandler,
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
    on_click: Option<ClickHandler>,
}

impl TaskRow {
    pub fn new(id: impl Into<ElementId>, data: TaskRowData) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            binding: None,
            on_click: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, data: TaskRowData, binding: Binding<bool>) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            binding: Some(binding),
            on_click: None,
        }
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
        let selected = binding.as_ref().map_or(self.selected, |b| b.get(cx));
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

        let has_click = binding.is_some() || handler.is_some();
        if has_click {
            row = row.on_click(move |event, window, cx| {
                if let Some(binding) = &binding {
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
