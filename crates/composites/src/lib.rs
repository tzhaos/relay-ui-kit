//! Relay composites layer for GPUI layout components.
//!
//! This layer depends on foundation and provides reusable shell, structure,
//! split, tab, and title-bar components without terminal or agent product
//! semantics.

pub mod layout;

pub use layout::{shell, structure};
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};
pub use structure::{KeyValue, ListSection, Tab, Tabs};
