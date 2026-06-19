use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, px, rgb};
use relay_ui_primitives::contract::{InputActionKind, InputValueKind};
use relay_ui_primitives::{
    Badge, Banner, Button, Callout, Checkbox, ColorPicker, ColorPreset, EmptyState, IconName,
    InlineError, LoadingSpinner, NumberInput, ProgressBar, Select, SelectOption, SettingsRow,
    SettingsSection, Skeleton, Slider, Theme, ThemePreviewCard, ThemePreviewKind, Toast, Toggle,
    Tone,
};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{scene_stack, section, strip, text_input_field},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
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
                        .description("Preview cards and select share the same host state")
                        .control(theme_controls(state, host)),
                )
                .row(
                    SettingsRow::new("Accent color")
                        .description("Preset picker emits the selected key and color")
                        .control(accent_picker(state, host)),
                )
                .row(
                    SettingsRow::new("UI font size")
                        .description("Stepper controls mutate gallery state")
                        .control(font_size_input(state, host, window)),
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
                                        this.state.launcher_choice = "settings";
                                        cx.notify();
                                    });
                                }
                            }),
                        ),
                )
                .child(
                    Callout::new("Shell path will be validated by the host")
                        .detail("The UI kit only renders the state; Relay should perform the actual command lookup before spawning a terminal.")
                        .tone(Tone::Info),
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

fn theme_select(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
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
        let open = state.settings_select_open;
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.settings_select_open = !open;
                cx.notify();
            });
        }
    })
    .on_select({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.theme_choice = key;
                this.state.settings_select_open = false;
                cx.notify();
            });
        }
    })
    .on_dismiss({
        let host = host.clone();
        move |_window, cx| {
            host.update(cx, |this, cx| {
                this.state.settings_select_open = false;
                cx.notify();
            });
        }
    })
}

fn theme_controls(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .gap_2()
        .child(
            div()
                .flex()
                .gap_2()
                .child(theme_card(
                    "settings-theme-system",
                    ThemePreviewKind::System,
                    state,
                    host,
                ))
                .child(theme_card(
                    "settings-theme-light",
                    ThemePreviewKind::Light,
                    state,
                    host,
                ))
                .child(theme_card(
                    "settings-theme-dark",
                    ThemePreviewKind::Dark,
                    state,
                    host,
                )),
        )
        .child(theme_select(state, host))
}

fn theme_card(
    id: &'static str,
    kind: ThemePreviewKind,
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
) -> impl IntoElement {
    ThemePreviewCard::new(id, kind)
        .selected(state.theme_choice == kind.key())
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.theme_choice = kind.key();
                    this.state.settings_select_open = false;
                    cx.notify();
                });
            }
        })
}

fn accent_picker(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    ColorPicker::new(
        "settings-accent-picker",
        state.accent_choice,
        vec![
            ColorPreset::new("green", "Green", rgb(0x16a34a).into()),
            ColorPreset::new("blue", "Blue", rgb(0x2563eb).into()),
            ColorPreset::new("violet", "Violet", rgb(0x7c3aed).into()),
            ColorPreset::new("amber", "Amber", rgb(0xb45309).into()),
            ColorPreset::new("red", "Red", rgb(0xb91c1c).into()),
        ],
    )
    .on_select({
        let host = host.clone();
        move |key, _color, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.accent_choice = key;
                cx.notify();
            });
        }
    })
}

fn font_size_input(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
) -> impl IntoElement {
    let focused = state.ui_font_size_focus.is_focused(window);

    NumberInput::new("settings-ui-font-size", state.ui_font_size)
        .input(state.ui_font_size_focus.clone(), &state.ui_font_size_input)
        .focused(focused)
        .suffix("px")
        .on_key({
            let host = host.clone();
            move |event, _window, cx| {
                host.update(cx, |this, cx| {
                    let action = this
                        .state
                        .ui_font_size_input
                        .handle_integer_key(event, false);
                    match action.contract_kind_for(InputValueKind::Number) {
                        InputActionKind::Changed(InputValueKind::Number)
                        | InputActionKind::Submit => {
                            sync_font_size_from_input(&mut this.state);
                            cx.notify();
                        }
                        InputActionKind::CursorMoved => cx.notify(),
                        InputActionKind::Cancel => {
                            sync_font_size_text(&mut this.state);
                            cx.notify();
                        }
                        InputActionKind::Ignored => {}
                        InputActionKind::Changed(_) | InputActionKind::Validate => {}
                    }
                });
            }
        })
        .on_decrement({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.ui_font_size = (this.state.ui_font_size - 1).max(11);
                    sync_font_size_text(&mut this.state);
                    cx.notify();
                });
            }
        })
        .on_increment({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.ui_font_size = (this.state.ui_font_size + 1).min(18);
                    sync_font_size_text(&mut this.state);
                    cx.notify();
                });
            }
        })
}

fn sync_font_size_from_input(state: &mut GalleryState) {
    if let Ok(value) = state.ui_font_size_input.value().parse::<i32>() {
        state.ui_font_size = value.clamp(11, 18);
    }
}

fn sync_font_size_text(state: &mut GalleryState) {
    state
        .ui_font_size_input
        .set_text(state.ui_font_size.to_string());
}

fn contrast_slider(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    Slider::new("settings-contrast", state.contrast, 0.0, 100.0)
        .on_change({
            let host = host.clone();
            move |value, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.contrast = value;
                    cx.notify();
                });
            }
        })
        .on_decrement({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.contrast = (this.state.contrast - 5.0).max(0.0);
                    cx.notify();
                });
            }
        })
        .on_increment({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.contrast = (this.state.contrast + 5.0).min(100.0);
                    cx.notify();
                });
            }
        })
}

fn notifications_toggle(on: bool, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    Checkbox::new("settings-notifications", on).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.notifications = !this.state.notifications;
                cx.notify();
            });
        }
    })
}

fn auto_archive_toggle(on: bool, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    Toggle::new("settings-auto-archive", on).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.auto_archive = !this.state.auto_archive;
                cx.notify();
            });
        }
    })
}
