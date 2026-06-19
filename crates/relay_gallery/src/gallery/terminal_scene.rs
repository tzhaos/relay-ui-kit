use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use relay_ui_primitives::{Button, IconButton, IconName, TextArea, TextInputAction, Theme};
use relay_workbench_ui::Composer;

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

    scene_stack()
        .child(section(
            cx,
            "Terminal session",
            terminal_sample(state, host, theme),
        ))
        .child(section(
            cx,
            "Session launcher",
            launcher_sample(state, host, theme),
        ))
        .child(section(
            cx,
            "Agent composer",
            composer_sample(state, host, window, theme),
        ))
        .child(section(cx, "Shell split", shell_split))
}

fn composer_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    theme: Theme,
) -> impl IntoElement {
    let composer_focused = state.composer_focus.is_focused(window);

    Composer::new(
        "terminal-composer",
        TextArea::new(
            "terminal-composer-input",
            state.composer_focus.clone(),
            &state.composer_input,
        )
        .placeholder("Ask an agent to work in the active terminal")
        .focused(composer_focused)
        .min_rows(3)
        .bordered(false)
        .on_key({
            let host = host.clone();
            move |event, _window, cx| {
                host.update(cx, |this, cx| {
                    match this.state.composer_input.handle_multiline_key(event) {
                        TextInputAction::Cancel => {
                            this.state.composer_input.clear();
                            cx.notify();
                        }
                        action if action.should_notify() => cx.notify(),
                        _ => {}
                    }
                });
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
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.state.terminal_session = "codex";
                        cx.notify();
                    });
                }
            }),
    )
}
