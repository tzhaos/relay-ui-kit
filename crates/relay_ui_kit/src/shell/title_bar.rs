use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Styled, Window, WindowControlArea, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, Theme, radius, space, ui_family};

/// A client-side title bar for windows opened without native decorations.
#[derive(IntoElement)]
pub struct TitleBar {
    app_name: String,
    project_label: Option<String>,
    center: Option<AnyElement>,
    actions: Option<AnyElement>,
    show_window_controls: bool,
}

impl TitleBar {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            project_label: None,
            center: None,
            actions: None,
            show_window_controls: true,
        }
    }

    pub fn project(mut self, project_label: impl Into<String>) -> Self {
        self.project_label = Some(project_label.into());
        self
    }

    pub fn center(mut self, center: impl IntoElement) -> Self {
        self.center = Some(center.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }

    pub fn window_controls(mut self, show: bool) -> Self {
        self.show_window_controls = show;
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .h(px(space::TITLE_BAR))
            .flex_shrink_0()
            .w_full()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(
                div()
                    .w(px(space::RAIL_WIDTH))
                    .h_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .size(px(22.0))
                            .rounded(px(radius::MD))
                            .bg(theme.accent)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.on_accent)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .child("R"),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(self.app_name),
                            )
                            .when_some(self.project_label, |this, project_label| {
                                this.child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_sm()
                                        .text_color(theme.text_muted)
                                        .child(project_label),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .h_full()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .window_control_area(WindowControlArea::Drag)
                    .when_some(self.center, |this, center| this.child(center)),
            )
            .child(
                div()
                    .h_full()
                    .flex_shrink_0()
                    .pl_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when_some(self.actions, |this, actions| this.child(actions))
                    .when(self.show_window_controls, |this| {
                        this.child(WindowControls::new())
                    }),
            )
    }
}

/// Native hit-test-backed window controls for client-decorated windows.
#[derive(IntoElement)]
pub struct WindowControls;

impl WindowControls {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowControls {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for WindowControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let maximize_kind = if window.is_maximized() {
            WindowControlKind::Restore
        } else {
            WindowControlKind::Maximize
        };

        div()
            .h_full()
            .flex()
            .items_center()
            .child(window_control_button(
                theme,
                WindowControlArea::Min,
                WindowControlKind::Minimize,
            ))
            .child(window_control_button(
                theme,
                WindowControlArea::Max,
                maximize_kind,
            ))
            .child(window_control_button(
                theme,
                WindowControlArea::Close,
                WindowControlKind::Close,
            ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowControlKind {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl WindowControlKind {
    fn glyph(self) -> &'static str {
        if cfg!(target_os = "windows") {
            return match self {
                WindowControlKind::Minimize => "\u{E921}",
                WindowControlKind::Maximize => "\u{E922}",
                WindowControlKind::Restore => "\u{E923}",
                WindowControlKind::Close => "\u{E8BB}",
            };
        }

        match self {
            WindowControlKind::Minimize => "\u{2212}",
            WindowControlKind::Maximize => "\u{25A1}",
            WindowControlKind::Restore => "\u{2750}",
            WindowControlKind::Close => "\u{00D7}",
        }
    }

    fn is_close(self) -> bool {
        self == WindowControlKind::Close
    }
}

fn window_control_button(
    theme: Theme,
    area: WindowControlArea,
    kind: WindowControlKind,
) -> gpui::Div {
    div()
        .w(px(44.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .font_family(window_control_font_family())
        .text_size(px(10.0))
        .line_height(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_muted)
        .window_control_area(area)
        .hover(move |style| {
            if kind.is_close() {
                style
                    .bg(theme.danger.opacity(0.08))
                    .text_color(theme.danger)
            } else {
                style.bg(theme.hover).text_color(theme.text_secondary)
            }
        })
        .child(kind.glyph())
}

fn window_control_font_family() -> &'static str {
    if cfg!(target_os = "windows") {
        "Segoe Fluent Icons"
    } else {
        ui_family()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_control_is_the_only_danger_control() {
        assert!(WindowControlKind::Close.is_close());
    }

    #[test]
    fn window_control_glyphs_are_not_empty() {
        assert!(!WindowControlKind::Minimize.glyph().is_empty());
    }
}
