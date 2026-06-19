use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use relay_ui_kit::{
    Badge, Banner, Button, Checkbox, ColorField, EmptyState, IconName, InlineError, LoadingSpinner,
    NumberInput, ProgressBar, Select, SelectOption, SettingsRow, SettingsSection, Skeleton, Slider,
    Theme, Toast, Toggle, Tone,
};

use super::{
    GalleryState,
    shared::{scene_stack, section, strip, text_input_field},
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let name_focused = state.name_focus.is_focused(window);
    let search_focused = state.search_focus.is_focused(window);

    scene_stack()
        .child(
            SettingsSection::new("Agent profile")
                .row(
                    SettingsRow::new("Agent name")
                        .description("Used as the default terminal session label")
                        .control(text_input_field(
                            host,
                            "settings-name",
                            &state.name_input,
                            state.name_focus.clone(),
                            name_focused,
                            None,
                            "Agent name",
                        )),
                )
                .row(
                    SettingsRow::new("Default file filter")
                        .description("Applied when opening review and file panels")
                        .control(text_input_field(
                            host,
                            "settings-filter",
                            &state.search_input,
                            state.search_focus.clone(),
                            search_focused,
                            Some(IconName::Search),
                            "Default file filter",
                        )),
                ),
        )
        .child(
            SettingsSection::new("Appearance")
                .row(
                    SettingsRow::new("Theme")
                        .description("Select follows the host-owned open/value state")
                        .control(theme_select(state, host)),
                )
                .row(
                    SettingsRow::new("Accent color")
                        .description("Color fields use a stable swatch plus value layout")
                        .control(ColorField::new(
                            "accent-color-field",
                            theme.accent,
                            "#339CFF",
                        )),
                )
                .row(
                    SettingsRow::new("UI font size")
                        .description("Stepper controls mutate gallery state")
                        .control(font_size_input(state, host)),
                )
                .row(
                    SettingsRow::new("Contrast")
                        .description("Slider exposes value and discrete step callbacks")
                        .control(contrast_slider(state, host)),
                ),
        )
        .child(
            SettingsSection::new("Behavior")
                .row(
                    SettingsRow::new("Notifications")
                        .description("Show task and terminal lifecycle notices")
                        .control(notifications_toggle(state.notifications, host)),
                )
                .row(
                    SettingsRow::new("Auto archive")
                        .description("Move completed sessions out of the active list")
                        .control(auto_archive_toggle(state.auto_archive, host)),
                ),
        )
        .child(section(
            cx,
            "Feedback",
            div()
                .max_w(px(520.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    Banner::new("Codex CLI is not available")
                        .detail("Install the CLI or update PATH before launching an agent.")
                        .tone(Tone::Warning)
                        .action(
                            Button::new("feedback-banner-action", "Open settings").on_click({
                                let host = host.clone();
                                move |_event, _window, cx| {
                                    host.update(cx, |this, cx| {
                                        this.gallery.launcher_choice = "settings";
                                        cx.notify();
                                    });
                                }
                            }),
                        ),
                )
                .child(
                    strip()
                        .child(Badge::new("RUNNING").tone(Tone::Accent).soft())
                        .child(Badge::new("WAITING").tone(Tone::Warning).soft())
                        .child(Badge::new("FAILED").tone(Tone::Danger).soft())
                        .child(Badge::new("main").tone(Tone::Secondary)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .flex_wrap()
                        .child(LoadingSpinner::new("feedback-spinner").label("Starting PTY"))
                        .child(
                            div().w(px(220.0)).child(
                                ProgressBar::new("feedback-progress", 6.0, 10.0)
                                    .label("Applying review comments"),
                            ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(Skeleton::new("feedback-skeleton-a").width(280.0))
                        .child(Skeleton::new("feedback-skeleton-b").width(220.0)),
                )
                .child(
                    InlineError::new("Agent launch failed")
                        .detail("The configured shell returned exit code 1."),
                )
                .child(
                    EmptyState::new(
                        "No saved agent profile",
                        "Create a profile to reuse defaults.",
                    )
                    .icon(IconName::Bot),
                )
                .child(
                    Toast::new("feedback-toast", "Terminal session restored")
                        .detail("codex on ui-kit/branch-controls")
                        .tone(Tone::Accent),
                )
                .child(div().text_xs().text_color(theme.text_muted).child(format!(
                    "Current launcher choice: {}",
                    state.launcher_choice
                ))),
        ))
}

fn theme_select(state: &GalleryState, host: &Entity<GalleryApp>) -> impl IntoElement {
    Select::new(
        "settings-theme-select",
        state.theme_choice,
        vec![
            SelectOption::new("system", "System").detail("Follow OS appearance"),
            SelectOption::new("light", "Light"),
            SelectOption::new("dark", "Dark"),
        ],
    )
    .open(state.settings_select_open)
    .on_toggle({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.settings_select_open = !this.gallery.settings_select_open;
                cx.notify();
            });
        }
    })
    .on_select({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.theme_choice = key;
                this.gallery.settings_select_open = false;
                cx.notify();
            });
        }
    })
}

fn font_size_input(state: &GalleryState, host: &Entity<GalleryApp>) -> impl IntoElement {
    NumberInput::new("settings-ui-font-size", state.ui_font_size)
        .suffix("px")
        .on_decrement({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.ui_font_size = (this.gallery.ui_font_size - 1).max(11);
                    cx.notify();
                });
            }
        })
        .on_increment({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.ui_font_size = (this.gallery.ui_font_size + 1).min(18);
                    cx.notify();
                });
            }
        })
}

fn contrast_slider(state: &GalleryState, host: &Entity<GalleryApp>) -> impl IntoElement {
    Slider::new("settings-contrast", state.contrast, 0.0, 100.0)
        .on_decrement({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.contrast = (this.gallery.contrast - 5.0).max(0.0);
                    cx.notify();
                });
            }
        })
        .on_increment({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.contrast = (this.gallery.contrast + 5.0).min(100.0);
                    cx.notify();
                });
            }
        })
}

fn notifications_toggle(on: bool, host: &Entity<GalleryApp>) -> impl IntoElement {
    Checkbox::new("settings-notifications", on).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.notifications = !this.gallery.notifications;
                cx.notify();
            });
        }
    })
}

fn auto_archive_toggle(on: bool, host: &Entity<GalleryApp>) -> impl IntoElement {
    Toggle::new("settings-auto-archive", on).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.auto_archive = !this.gallery.auto_archive;
                cx.notify();
            });
        }
    })
}
