use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay::{Binding, Selector};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ActionHandler, SelectionBinding},
    theme::{ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY, radius},
};

use super::KeybindingShortcut;

/// One command row inside a command palette or launcher.
#[derive(IntoElement)]
pub struct CommandRow<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    key: K,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
    shortcut: Option<KeybindingShortcut>,
    selected: bool,
    disabled: bool,
    selection: Option<SelectionBinding>,
    on_select: Option<ActionHandler<K>>,
}

impl<K> CommandRow<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    pub fn new(id: impl Into<ElementId>, key: K, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            selection: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        key: K,
        label: impl Into<String>,
        binding: Binding<K>,
    ) -> Self {
        let selection = SelectionBinding::binding(binding, key.clone());

        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            selection: Some(selection),
            on_select: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: KeybindingShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn selected_with(mut self, selection: SelectionBinding) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn selected_by(self, selector: Selector<K>) -> Self {
        let key = self.key.clone();
        self.selected_with(SelectionBinding::selector(selector, key))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl<K> RenderOnce for CommandRow<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selection = self.selection;
        let selected = selection
            .as_ref()
            .map_or(self.selected, |selection| selection.is_selected(cx));
        let handler = self.on_select;
        let disabled = self.disabled || (handler.is_none() && selection.is_none());
        let key = self.key;

        div()
            .id(self.id)
            .min_h(px(44.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .when(selected, |this| this.bg(theme.selection))
            .when(disabled, |this| this.opacity(DISABLED_OPACITY))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    div()
                        .size(px(24.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(icon)
                                .size(IconSize::Small)
                                .color(theme.text_secondary),
                        ),
                )
            })
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
                            .child(self.label),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .when_some(self.shortcut, |this, shortcut| this.child(shortcut))
            .when(!disabled, |this| {
                this.on_click(move |_: &ClickEvent, window, cx| {
                    if let Some(selection) = &selection {
                        selection.select(cx);
                    }
                    if let Some(handler) = &handler {
                        handler(key.clone(), window, cx);
                    }
                    cx.stop_propagation();
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{ReactiveAppExt, SelectionModel, init};

    use super::*;

    fn row_selection<K>(row: &CommandRow<K>) -> &SelectionBinding
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let Some(selection) = row.selection.as_ref() else {
            panic!("row should store selection");
        };
        selection
    }

    #[test]
    fn command_row_selected_by_reads_selector_key() {
        let mut app = TestApp::new();
        let row = app.update(|cx| {
            init(cx);
            let selector = cx.selector(Some("open"));
            CommandRow::new("command-open", "open", "Open").selected_by(selector)
        });

        app.update(|cx| {
            let selection = row_selection(&row);
            assert!(selection.is_selected(cx));
        });
    }

    #[test]
    fn command_row_selection_binding_selects_row_key() {
        let mut app = TestApp::new();
        let (selector, row) = app.update(|cx| {
            init(cx);
            let selector = cx.selector(Some("open"));
            let row =
                CommandRow::new("command-close", "close", "Close").selected_by(selector.clone());
            (selector, row)
        });

        app.update(|cx| {
            let selection = row_selection(&row);
            selection.select(cx);
        });

        app.read(|cx| {
            assert_eq!(selector.get(cx), Some("close"));
        });
    }

    #[test]
    fn command_row_selected_with_selection_model_selects_row_key() {
        let mut app = TestApp::new();
        let (selection, row) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("open"));
            let row = CommandRow::new("command-close", "close", "Close").selected_with(
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
