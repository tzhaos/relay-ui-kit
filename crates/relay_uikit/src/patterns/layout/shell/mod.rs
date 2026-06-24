//! Workbench shell chrome and pane layout infrastructure.
//!
//! The shell layer is responsible for the reusable frame around product
//! content:
//!
//! - [`AppShell`] for top-level application composition;
//! - [`Pane`], [`PaneSurface`], and [`PaneWidth`] for bounded content regions;
//! - [`SplitPane`] and [`SplitPaneState`] for resizable multi-pane layout;
//! - [`TitleBar`], [`WindowControls`], [`PaneToolbar`], and [`TopToolbar`] for
//!   desktop chrome;
//! - [`StatusBar`] and [`StatusItem`] for bottom-strip feedback and actions.
//!
//! This layer should stay focused on geometry, chrome, and focus continuity.
//! It should not own domain workflow. Hosts are still responsible for what each
//! pane means, what data it shows, and how that data is loaded or mutated.
//!
//! A shell component is only considered finished when it remains stable under
//! rapid resize, pane switching, and retained child state churn.

mod app_shell;
mod pane;
mod split_pane;
mod status_bar;
mod title_bar;
mod toolbar;

pub use app_shell::AppShell;
pub use pane::{Pane, PaneSurface, PaneWidth};
pub use split_pane::{SplitAxis, SplitPane, SplitPaneState};
pub use status_bar::{StatusBar, StatusItem};
pub use title_bar::{TitleBar, WindowControls};
pub use toolbar::{PaneToolbar, TopToolbar, WorkspaceBreadcrumb};
