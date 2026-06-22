use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use relay::{Binding, Selector};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{SelectHandler, SelectionBinding},
    theme::{radius, ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY},
};

use super::KeybindingShortcut;

/// One command row inside a command palette or launcher.
#[derive(IntoElement)]
pub struct CommandRow {
    id: ElementId,
    key: &'static str,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
    shortcut: Option<KeybindingShortcut>,
    selected: bool,
    disabled: bool,
    binding: Option<Binding<&'static str>>,
    selection: Option<SelectionBinding>,
    on_select: Option<SelectHandler>,
}

impl CommandRow {
    pub fn new(id: impl Into<ElementId>, key: &'static str, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            binding: None,
            selection: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        key: &'static str,
        label: impl Into<String>,
        binding: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            binding: Some(binding),
            selection: None,
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

    pub fn selected_by(mut self, selector: Selector<&'static str>) -> Self {
        self.selection = Some(SelectionBinding::selector(selector, self.key));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CommandRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let selection = self.selection;
        let selected = if let Some(selection) = &selection {
            selection.is_selected(cx)
        } else {
            binding
                .as_ref()
                .map_or(self.selected, |b| b.get(cx) == self.key)
        };
        let handler = self.on_select;
        let disabled =
            self.disabled || (handler.is_none() && binding.is_none() && selection.is_none());
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
                    } else if let Some(binding) = &binding {
                        binding.set(cx, key);
                    }
                    if let Some(handler) = &handler {
                        handler(key, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{init, ReactiveAppExt};

    use super::*;

    #[test]
    fn command_row_selected_by_reads_selector_key() {
        let mut app = TestApp::new();
        let row = app.update(|cx| {
            init(cx);
            let selector = cx.selector(Some("open"));
            CommandRow::new("command-open", "open", "Open").selected_by(selector)
        });

        app.update(|cx| {
            let selection = row.selection.as_ref().expect("row should store selection");
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
            let selection = row.selection.as_ref().expect("row should store selection");
            selection.select(cx);
        });

        app.read(|cx| {
            assert_eq!(selector.get(cx), Some("close"));
        });
    }
}
