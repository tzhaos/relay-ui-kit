use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay::{Binding, Selector, WindowSignalExt};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ActionHandler, SelectionSource},
    theme::ActiveTheme,
};

/// One tab in a [`Tabs`] bar.
pub struct Tab<K> {
    key: K,
    label: String,
    icon: Option<IconName>,
    count: Option<usize>,
}

impl<K> Tab<K> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            icon: None,
            count: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

/// An underline tab bar.
#[derive(IntoElement)]
pub struct Tabs<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    tabs: Vec<Tab<K>>,
    active: Option<K>,
    selection: Option<SelectionSource<K>>,
    on_select: Option<ActionHandler<K>>,
}

impl<K> Tabs<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    pub fn new(id: impl Into<ElementId>, tabs: Vec<Tab<K>>) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: None,
            selection: None,
            on_select: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, tabs: Vec<Tab<K>>, binding: Binding<K>) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: None,
            selection: Some(SelectionSource::binding(binding)),
            on_select: None,
        }
    }

    pub fn active(mut self, active: K) -> Self {
        self.active = Some(active);
        self
    }

    pub fn selected_with(mut self, selection: SelectionSource<K>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn selected_by(mut self, selector: Selector<K>) -> Self {
        self.selection = Some(SelectionSource::selector(selector));
        self
    }

    pub fn on_select(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl<K> RenderOnce for Tabs<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            tabs,
            active,
            selection,
            on_select,
        } = self;
        let selection = selection.or_else(|| {
            active.clone().map(|active| {
                SelectionSource::binding(
                    window.use_binding((id.clone(), "active-tab"), cx, || active),
                )
            })
        });
        let active = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(active);
        let handler = on_select.map(std::rc::Rc::new);
        let mut row = div()
            .id(id)
            .h(px(36.0))
            .w_full()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(theme.border);

        for (index, tab) in tabs.into_iter().enumerate() {
            let is_active = active.as_ref().is_some_and(|active| tab.key == *active);
            let key = tab.key.clone();
            let handler = handler.clone();
            let selection = selection.clone();
            let (fg, underline) = if is_active {
                (theme.text, theme.accent)
            } else {
                (theme.text_muted, gpui::transparent_black())
            };
            let clickable = !is_active && (selection.is_some() || handler.is_some());

            let cell = div()
                .id(("tab", index))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(fg)
                .border_b_2()
                .border_color(underline)
                .when(clickable, |this| {
                    this.cursor_pointer()
                        .hover(move |s| s.text_color(theme.text_secondary))
                })
                .when_some(tab.icon, |this, icon| {
                    let color = if is_active {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    this.child(Icon::new(icon).size(IconSize::Small).color(color))
                })
                .child(tab.label)
                .when_some(tab.count, |this, count| {
                    this.child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(format!("({count})")),
                    )
                })
                .when(clickable, |this| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        if let Some(selection) = &selection {
                            selection.select(cx, key.clone());
                        }
                        if let Some(handler) = &handler {
                            handler(key.clone(), window, cx);
                        }
                        cx.stop_propagation();
                    })
                });
            row = row.child(cell);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{SelectionModel, init};

    use super::*;

    fn current_active_key<K>(tabs: &Tabs<K>, cx: &App) -> Option<K>
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        tabs.selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or_else(|| tabs.active.clone())
    }

    #[test]
    fn tabs_reads_active_key_from_selection_source() {
        let mut app = TestApp::new();
        let selection = app.update(|cx| {
            init(cx);
            SelectionModel::new(cx, Some("files"))
        });
        let tabs = Tabs::new(
            "tabs",
            vec![Tab::new("files", "Files"), Tab::new("review", "Review")],
        )
        .selected_with(SelectionSource::selection_model(selection.clone()));

        let initial = app.read(|cx| current_active_key(&tabs, cx));
        assert_eq!(initial, Some("files"));

        app.update(|cx| {
            selection.select(cx, "review");
        });

        let updated = app.read(|cx| current_active_key(&tabs, cx));
        assert_eq!(updated, Some("review"));
    }
}
