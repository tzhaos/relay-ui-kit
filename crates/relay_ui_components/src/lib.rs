//! Composed GPUI layout components for Relay UI.
//!
//! This layer depends on primitives and provides reusable shell, structure,
//! split, scroll, tab, and title-bar components without terminal or agent
//! product semantics.

pub mod layout;

pub use relay_ui_primitives::{
    ActiveTheme, Icon, IconName, IconSize, KitAssets, MotionDirection, MotionDuration, MotionExt,
    Theme, Tone, button, choice, command, components, controls, display, feedback, form, icon,
    input, interaction, list, motion, radius, row, space, theme, tone,
};

pub use layout::{shell, structure};
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};
pub use structure::{KeyValue, ListSection, ScrollSurface, Tab, Tabs};
