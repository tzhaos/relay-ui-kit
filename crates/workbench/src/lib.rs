//! Relay workbench UI compositions.
//!
//! This layer is allowed to speak in Relay product terms such as terminals,
//! agents, git branches, launchers, and read-only file viewers.

pub mod workbench;

pub use workbench::{composer, git, launcher, terminal, viewer};

pub use composer::Composer;
pub use git::{
    BranchActionKind, BranchActionsMenu, BranchOption, BranchPickerAction, BranchSelector,
};
pub use launcher::{LauncherItem, LauncherItemKind, LauncherMenu};
#[allow(deprecated)]
pub use terminal::AgentQuickLaunch;
pub use terminal::{
    TerminalAgentQuickLaunch, TerminalLine, TerminalLineStyle, TerminalSessionRow,
    TerminalStatusBadge, TerminalSurface, TerminalTab, TerminalToolbar, TerminalTranscript,
};
pub use viewer::{
    CodeView, DiffHunk, DiffLine, DiffLineKind, DiffView, FileKind, FileView, MarkdownView,
};
pub use workbench::task_row::{TaskRow, TaskRowData};
