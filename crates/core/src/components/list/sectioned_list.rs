use crate::component_prelude::*;

pub struct SectionedListGroup {
    title: String,
    count: Option<usize>,
    trailing: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl SectionedListGroup {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            count: None,
            trailing: None,
            children: Vec::new(),
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
}

impl ParentElement for SectionedListGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

#[derive(IntoElement)]
pub struct SectionedList {
    id: ElementId,
    groups: Vec<SectionedListGroup>,
}

impl SectionedList {
    pub fn new(id: impl Into<ElementId>, groups: Vec<SectionedListGroup>) -> Self {
        Self {
            id: id.into(),
            groups,
        }
    }
}

impl RenderOnce for SectionedList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id(self.id)
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2()
            .children(
                self.groups
                    .into_iter()
                    .map(move |group| group.render(theme)),
            )
    }
}

impl SectionedListGroup {
    fn render(self, theme: crate::Theme) -> impl IntoElement {
        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(space::ROW_SM))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text_muted)
                                    .child(self.title.to_uppercase()),
                            )
                            .when_some(self.count, |this, count| {
                                this.child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text_muted)
                                        .child(count.to_string()),
                                )
                            }),
                    )
                    .when_some(self.trailing, |this, trailing| this.child(trailing)),
            )
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.children),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sectioned_list_group_starts_without_rows() {
        let group = SectionedListGroup::new("Recent");

        assert!(group.children.is_empty());
    }
}
