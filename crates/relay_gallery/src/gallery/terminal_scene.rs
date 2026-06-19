use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div};
use relay_ui_kit::{Button, Composer, IconButton, IconName, TextInput, TextInputAction, Theme};

use super::{
    GalleryState,
    product_samples::{launcher_sample, shell_sample, terminal_sample},
    shared::{scene_stack, section},
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
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
            composer_sample(state, host, theme),
        ))
        .child(section(cx, "Shell split", shell_sample(state, host)))
}

fn composer_sample(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    theme: Theme,
) -> impl IntoElement {
    Composer::new(
        "terminal-composer",
        TextInput::new(
            "terminal-composer-input",
            state.search_focus.clone(),
            &state.search_input,
        )
        .placeholder("Ask an agent to work in the active terminal")
        .leading_icon(IconName::Bot)
        .on_key({
            let host = host.clone();
            move |event, _window, cx| {
                host.update(cx, |this, cx| {
                    match this.gallery.search_input.handle_key(event) {
                        TextInputAction::Edited | TextInputAction::Submit => cx.notify(),
                        TextInputAction::Cancel => {
                            this.gallery.search_input.clear();
                            cx.notify();
                        }
                        TextInputAction::Ignored => {}
                    }
                });
            }
        }),
    )
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
    .trailing(Button::new("composer-start", "Start").primary().on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.terminal_session = "codex";
                cx.notify();
            });
        }
    }))
}
