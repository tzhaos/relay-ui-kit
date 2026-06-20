use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, WindowControlArea, div, px, rgb,
};

use relay_ui_core::theme::{ActiveTheme, Theme, ui_family};

fn windows_close_hover() -> Hsla {
    rgb(0xe81120).into()
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
    fn id(self) -> &'static str {
        match self {
            WindowControlKind::Minimize => "window-control-minimize",
            WindowControlKind::Maximize => "window-control-maximize",
            WindowControlKind::Restore => "window-control-restore",
            WindowControlKind::Close => "window-control-close",
        }
    }

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
) -> impl IntoElement {
    div()
        .id(kind.id())
        .w(px(36.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .font_family(window_control_font_family())
        .text_size(px(10.0))
        .text_color(window_control_foreground(theme, kind))
        .window_control_area(area)
        .hover(move |style| {
            style
                .bg(window_control_hover_background(theme, kind))
                .text_color(window_control_hover_foreground(theme, kind))
        })
        .active(move |style| {
            style
                .bg(window_control_active_background(theme, kind))
                .text_color(window_control_active_foreground(theme, kind))
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

fn window_control_hover_background(theme: Theme, kind: WindowControlKind) -> Hsla {
    if kind.is_close() {
        windows_close_hover()
    } else {
        theme.hover
    }
}

fn window_control_hover_foreground(theme: Theme, kind: WindowControlKind) -> Hsla {
    if kind.is_close() {
        gpui::white()
    } else {
        theme.text_secondary
    }
}

fn window_control_foreground(theme: Theme, _kind: WindowControlKind) -> Hsla {
    theme.text_muted
}

fn window_control_active_background(theme: Theme, kind: WindowControlKind) -> Hsla {
    window_control_hover_background(theme, kind).opacity(0.86)
}

fn window_control_active_foreground(theme: Theme, kind: WindowControlKind) -> Hsla {
    window_control_hover_foreground(theme, kind).opacity(if kind.is_close() { 0.92 } else { 1.0 })
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

    #[test]
    fn window_control_ids_are_stable() {
        assert_eq!(WindowControlKind::Close.id(), "window-control-close");
    }

    #[test]
    fn close_control_hover_uses_system_red() {
        let theme = Theme::light();

        assert_eq!(
            window_control_hover_background(theme, WindowControlKind::Close),
            windows_close_hover()
        );
    }

    #[test]
    fn close_control_hover_uses_high_contrast_foreground() {
        let theme = Theme::light();

        assert_eq!(
            window_control_hover_foreground(theme, WindowControlKind::Close),
            gpui::white()
        );
    }

    #[test]
    fn close_control_active_keeps_high_contrast_foreground() {
        let theme = Theme::light();

        assert_eq!(
            window_control_active_foreground(theme, WindowControlKind::Close),
            gpui::white().opacity(0.92)
        );
    }
}
