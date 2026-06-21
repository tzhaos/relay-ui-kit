use gpui::{
    Anchor, Context, Entity, IntoElement, ParentElement, Styled, Window, div,
    prelude::FluentBuilder, px, rgb,
};
use relay::SignalVecExt;
use relay_uikit::patterns::overlay::{Select, SelectOption, overlay};
use relay_uikit::{
    Badge, Banner, Button, Callout, Checkbox, ColorPicker, ColorPreset, EmptyState, IconName,
    InlineError, LoadingSpinner, NumberInput, ProgressBar, SettingsRow, SettingsSection, Skeleton,
    Slider, Theme, ThemePreviewCard, ThemePreviewKind, Toast, Toggle, Tone, radius,
};

use super::{
    FEEDBACK_TOAST_DURATION, GalleryScenesApp, GalleryState,
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
    let settings_dirty = state.settings_dirty.get(cx);

    scene_stack()
        .when(settings_dirty, |this| {
            this.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded(px(radius::LG))
                    .bg(theme.panel_alt)
                    .border_1()
                    .border_color(theme.accent.opacity(0.4))
                    .text_sm()
                    .text_color(theme.accent)
                    .child("Unsaved changes — derived via Form::is_dirty"),
            )
        })
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
                        .control(theme_controls(state, cx)),
                )
                .row(
                    SettingsRow::new("Accent color")
                        .description("Preset picker emits the selected key and color")
                        .control(accent_picker(state, cx)),
                )
                .row(
                    SettingsRow::new("UI font size")
                        .description("Stepper controls mutate gallery state")
                        .control(font_size_input(state, window)),
                )
                .row(
                    SettingsRow::new("Contrast")
                        .description("Slider exposes value and discrete step callbacks")
                        .control(contrast_slider(state)),
                ),
        )
        .child(
            SettingsSection::new("Behavior")
                .row(
                    SettingsRow::new("Notifications")
                        .description("Show task and terminal lifecycle notices")
                        .control(notifications_toggle(state)),
                )
                .row(
                    SettingsRow::new("Auto archive")
                        .description("Move completed sessions out of the active list")
                        .control(auto_archive_toggle(state)),
                ),
        )
        .child(section(
            cx,
            "Feedback",
            div()
                .relative()
                .w_full()
                .max_w(px(720.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    Banner::new("Codex CLI is not available")
                        .detail("Install the CLI or update PATH before launching an agent.")
                        .tone(Tone::Warning)
                        .action(
                            Button::new("feedback-banner-action", "Open settings").on_click({
                                let seg_tab = state.seg_tab.clone();
                                move |_event, _window, cx| {
                                    seg_tab.set(cx, "settings");
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
                    strip()
                        .child(
                            Button::new(
                                "feedback-show-toast",
                                "Add toast",
                            )
                            .icon(IconName::MessageSquareText)
                            .on_click({
                                let host = host.clone();
                                move |_event, _window, cx| {
                                    host.update(cx, |this, cx| {
                                        this.add_feedback_toast(cx, "Notification sample");
                                    });
                                }
                            }),
                        )
                        .child(
                            Button::new("feedback-hide-toast", "Clear")
                                .ghost()
                                .disabled(state.feedback_toasts.read(cx, |t| t.is_empty()))
                                .on_click({
                                    let toasts = state.feedback_toasts.clone();
                                    move |_event, _window, cx| {
                                        // SignalVecExt::clear notifies the view
                                        // automatically — no cx.notify() needed.
                                        toasts.clear(cx);
                                    }
                                }),
                        ),
                )
                .when(!state.feedback_toasts.read(cx, |t| t.is_empty()), |this| {
                    let toasts = state.feedback_toasts.get(cx);
                    this.child(
                        overlay(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .gap_2()
                                .children(toasts.iter().map(|toast| {
                                    let id = toast.id;
                                    let msg = toast.message.clone();
                                    Toast::new(
                                        format!("feedback-floating-toast-{id}"),
                                        msg,
                                    )
                                    .detail(format!("Dismisses in {:.0}s", FEEDBACK_TOAST_DURATION.as_secs()))
                                    .tone(Tone::Accent)
                                    .on_close({
                                        let host = host.clone();
                                        move |_event, _window, cx| {
                                            host.update(cx, |this, cx| {
                                                // dismiss_feedback_toast now takes
                                                // cx and notifies via the signal.
                                                this.dismiss_feedback_toast(id, cx);
                                            });
                                        }
                                    })
                                })),
                        )
                        .window_corner(Anchor::BottomRight, 16.0)
                    )
                })
                .child(div().text_xs().text_color(theme.text_muted).child(format!(
                    "Current tab: {}",
                    state.seg_tab.get(cx)
                ))),
        ))
}

fn theme_select(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let open_binding = state.settings_select_open.clone();
    let is_open = open_binding.get(cx);

    Select::bound(
        "settings-theme-select",
        state.theme_choice.clone(),
        vec![
            SelectOption::new("system", "System").detail("Follow OS appearance"),
            SelectOption::new("light", "Light"),
            SelectOption::new("dark", "Dark"),
        ],
    )
    .open(is_open)
    .on_toggle({
        let open = open_binding.clone();
        move |_event, _window, cx| {
            open.update(cx, |v| {
                *v = !*v;
                true
            });
        }
    })
    .on_select({
        let open = open_binding.clone();
        move |_key, _window, cx| {
            open.set(cx, false);
        }
    })
    .on_dismiss({
        move |_window, cx| {
            open_binding.set(cx, false);
        }
    })
}

fn theme_controls(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .gap_2()
        .child(
            div()
                .flex()
                .gap_2()
                .child(ThemePreviewCard::bound(
                    "settings-theme-system",
                    ThemePreviewKind::System,
                    state.theme_choice.clone(),
                ))
                .child(ThemePreviewCard::bound(
                    "settings-theme-light",
                    ThemePreviewKind::Light,
                    state.theme_choice.clone(),
                ))
                .child(ThemePreviewCard::bound(
                    "settings-theme-dark",
                    ThemePreviewKind::Dark,
                    state.theme_choice.clone(),
                )),
        )
        .child(theme_select(state, cx))
}

fn accent_picker(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let accent_choice = state.accent_choice.clone();
    ColorPicker::new("settings-accent-picker", accent_choice.get(cx), vec![
        ColorPreset::new("green", "Green", rgb(0x16a34a).into()),
        ColorPreset::new("blue", "Blue", rgb(0x2563eb).into()),
        ColorPreset::new("violet", "Violet", rgb(0x7c3aed).into()),
        ColorPreset::new("amber", "Amber", rgb(0xb45309).into()),
        ColorPreset::new("red", "Red", rgb(0xb91c1c).into()),
    ])
    .on_select({
        let accent_choice = accent_choice.clone();
        move |key, _hsla, _window, cx| {
            accent_choice.set(cx, key);
        }
    })
}

fn font_size_input(state: &GalleryState, window: &Window) -> impl IntoElement {
    let focused = state.ui_font_size_focus.is_focused(window);

    NumberInput::bound("settings-ui-font-size", state.ui_font_size.clone())
        .input_bound(
            state.ui_font_size_focus.clone(),
            state.ui_font_size_input.clone(),
        )
        .focused(focused)
        .range(11, 18)
        .suffix("px")
}

fn contrast_slider(state: &GalleryState) -> impl IntoElement {
    Slider::bound("settings-contrast", state.contrast.clone(), 0.0, 100.0)
        .on_decrement({
            let contrast = state.contrast.clone();
            move |_event, _window, cx| {
                let value = (contrast.get(cx) - 5.0).max(0.0);
                contrast.set(cx, value);
            }
        })
        .on_increment({
            let contrast = state.contrast.clone();
            move |_event, _window, cx| {
                let value = (contrast.get(cx) + 5.0).min(100.0);
                contrast.set(cx, value);
            }
        })
}

fn notifications_toggle(state: &GalleryState) -> impl IntoElement {
    Checkbox::bound("settings-notifications", state.notifications.clone())
}

fn auto_archive_toggle(state: &GalleryState) -> impl IntoElement {
    Toggle::bound("settings-auto-archive", state.auto_archive.clone())
}
