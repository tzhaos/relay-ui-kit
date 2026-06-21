use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use relay_uikit::workbench::Composer;
use relay_uikit::{Button, IconButton, IconName, TextArea, Theme};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{scene_stack, section},
    workbench_samples::{launcher_sample, shell_sample, terminal_sample},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let shell_split = shell_sample(state, host, cx);
    let terminal_body = terminal_sample(state, host, theme, cx);
    let launcher_body = launcher_sample(state, host, theme, cx);

    scene_stack()
        .child(section(cx, "Terminal session", terminal_body))
        .child(section(cx, "Session launcher", launcher_body))
        .child(section(
            cx,
            "Agent composer",
            composer_sample(state, host, window, theme),
        ))
        .child(section(cx, "Shell split", shell_split))
}

fn composer_sample(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    window: &Window,
    theme: Theme,
) -> impl IntoElement {
    let composer_focused = state.composer_focus.is_focused(window);

    Composer::new(
        "terminal-composer",
        TextArea::bound(
            "terminal-composer-input",
            state.composer_focus.clone(),
            state.composer_input.clone(),
        )
        .placeholder("Ask an agent to work in the active terminal")
        .focused(composer_focused)
        .min_rows(3)
        .bordered(false)
        .on_key({
            let composer_input = state.composer_input.clone();
            move |event, _window, cx| {
                // Handle Cancel specially — clear the input on Escape.
                if event.keystroke.key.as_str() == "escape" {
                    composer_input.update(cx, |s| {
                        s.clear();
                        true
                    });
                }
            }
        }),
    )
    .floating(true)
    .leading(
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(IconButton::new("composer-attach", IconName::Plus))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Active terminal"),
            ),
    )
    .trailing(
        Button::new("composer-start", "Start")
            .primary()
            .icon(IconName::Play)
            .on_click({
                let terminal_session = state.terminal_session.clone();
                move |_event, _window, cx| {
                    terminal_session.set(cx, "codex");
                }
            }),
    )
}
