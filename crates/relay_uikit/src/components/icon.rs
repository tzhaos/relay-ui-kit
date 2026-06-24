//! Icon system for the Relay UI kit.
//!
//! Icons are Lucide SVGs embedded at compile time. GPUI's `svg()` element tints
//! the glyph with the element's `text_color`, so an [`Icon`] is just a sized,
//! colored SVG whose path resolves through the [`KitAssets`] asset source.
//!
//! The Lucide set uses `stroke="currentColor"`, which GPUI maps to the resolved
//! `text_color` — so an icon inherits tone exactly like text does.

use std::borrow::Cow;

use gpui::{App, AssetSource, IntoElement, Result, SharedString, Styled, Window, px, svg};

use crate::theme::ActiveTheme;

/// Compile-time embedded Lucide icon set.
///
/// Each variant maps to a file under `assets/icons/<name>.svg`. Keep this list
/// aligned with the files actually present; [`IconName::path`] derives the asset
/// path from the variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    Archive,
    ArrowLeft,
    ArrowRight,
    Bot,
    CalendarClock,
    Check,
    ChevronDown,
    ChevronRight,
    CircleDot,
    Ellipsis,
    FileDiff,
    FileText,
    Folder,
    FolderPlus,
    Funnel,
    GitBranch,
    LayoutGrid,
    List,
    ListChecks,
    ListFilter,
    MessageSquareText,
    Minus,
    PanelLeft,
    Pencil,
    Play,
    Plus,
    RefreshCw,
    RotateCcw,
    Search,
    Settings,
    SlidersHorizontal,
    Terminal,
    Trash2,
    X,
    Zap,
}

impl IconName {
    /// The asset path for this icon, e.g. `icons/terminal.svg`.
    pub fn path(self) -> SharedString {
        let file = match self {
            IconName::Archive => "archive",
            IconName::ArrowLeft => "arrow-left",
            IconName::ArrowRight => "arrow-right",
            IconName::Bot => "bot",
            IconName::CalendarClock => "calendar-clock",
            IconName::Check => "check",
            IconName::ChevronDown => "chevron-down",
            IconName::ChevronRight => "chevron-right",
            IconName::CircleDot => "circle-dot",
            IconName::Ellipsis => "ellipsis",
            IconName::FileDiff => "file-diff",
            IconName::FileText => "file-text",
            IconName::Folder => "folder",
            IconName::FolderPlus => "folder-plus",
            IconName::Funnel => "funnel",
            IconName::GitBranch => "git-branch",
            IconName::LayoutGrid => "layout-grid",
            IconName::List => "list",
            IconName::ListChecks => "list-checks",
            IconName::ListFilter => "list-filter",
            IconName::MessageSquareText => "message-square-text",
            IconName::Minus => "minus",
            IconName::PanelLeft => "panel-left",
            IconName::Pencil => "pencil",
            IconName::Play => "play",
            IconName::Plus => "plus",
            IconName::RefreshCw => "refresh-cw",
            IconName::RotateCcw => "rotate-ccw",
            IconName::Search => "search",
            IconName::Settings => "settings",
            IconName::SlidersHorizontal => "sliders-horizontal",
            IconName::Terminal => "terminal",
            IconName::Trash2 => "trash-2",
            IconName::X => "x",
            IconName::Zap => "zap",
        };
        format!("icons/{file}.svg").into()
    }

    /// A generic human-readable label for icon-only affordances.
    ///
    /// Hosts should still provide a more specific label when the action depends
    /// on surrounding product context, but this keeps icon-only controls from
    /// falling back to an unlabeled state.
    pub fn label(self) -> &'static str {
        match self {
            IconName::Archive => "Archive",
            IconName::ArrowLeft => "Previous",
            IconName::ArrowRight => "Next",
            IconName::Bot => "Assistant",
            IconName::CalendarClock => "Schedule",
            IconName::Check => "Confirm",
            IconName::ChevronDown => "Open menu",
            IconName::ChevronRight => "Open submenu",
            IconName::CircleDot => "Selected",
            IconName::Ellipsis => "More actions",
            IconName::FileDiff => "Diff",
            IconName::FileText => "File",
            IconName::Folder => "Folder",
            IconName::FolderPlus => "New folder",
            IconName::Funnel => "Filter",
            IconName::GitBranch => "Git branch",
            IconName::LayoutGrid => "Grid",
            IconName::List => "List",
            IconName::ListChecks => "Tasks",
            IconName::ListFilter => "Filtered list",
            IconName::MessageSquareText => "Conversation",
            IconName::Minus => "Remove",
            IconName::PanelLeft => "Toggle panel",
            IconName::Pencil => "Edit",
            IconName::Play => "Run",
            IconName::Plus => "Add",
            IconName::RefreshCw => "Refresh",
            IconName::RotateCcw => "Reset",
            IconName::Search => "Search",
            IconName::Settings => "Settings",
            IconName::SlidersHorizontal => "Adjust controls",
            IconName::Terminal => "Terminal",
            IconName::Trash2 => "Delete",
            IconName::X => "Close",
            IconName::Zap => "Quick action",
        }
    }
}

/// Asset source backing the kit's icons. Embeds every SVG under
/// `assets/icons/` into the binary so no runtime file IO is needed.
#[derive(Clone)]
pub struct KitAssets;

macro_rules! icon_assets {
    ($($file:literal),* $(,)?) => {
        impl AssetSource for KitAssets {
            fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
                match path {
                    $(
                        concat!("icons/", $file, ".svg") => Ok(Some(Cow::Borrowed(
                            include_bytes!(concat!("../../assets/icons/", $file, ".svg")).as_slice(),
                        ))),
                    )*
                    _ => Ok(None),
                }
            }

            fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
                Ok(vec![])
            }
        }
    };
}

icon_assets![
    "archive",
    "arrow-left",
    "arrow-right",
    "bot",
    "calendar-clock",
    "check",
    "chevron-down",
    "chevron-right",
    "circle-dot",
    "ellipsis",
    "file-diff",
    "file-text",
    "folder",
    "folder-plus",
    "funnel",
    "git-branch",
    "layout-grid",
    "list",
    "list-checks",
    "list-filter",
    "message-square-text",
    "minus",
    "panel-left",
    "pencil",
    "play",
    "plus",
    "refresh-cw",
    "rotate-ccw",
    "search",
    "settings",
    "sliders-horizontal",
    "terminal",
    "trash-2",
    "x",
    "zap"
];

/// Visual size presets for [`Icon`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    /// 12px — inline metadata.
    XSmall,
    /// 14px — dense rows, badges.
    Small,
    /// 16px — default UI icon.
    Medium,
    /// 20px — nav rail, toolbar emphasis.
    Large,
}

impl IconSize {
    pub fn px(self) -> f32 {
        match self {
            IconSize::XSmall => 12.0,
            IconSize::Small => 14.0,
            IconSize::Medium => 16.0,
            IconSize::Large => 20.0,
        }
    }
}

/// A sized, tinted icon element.
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: IconSize,
    color: Option<gpui::Hsla>,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: IconSize::Medium,
            color: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: gpui::Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl gpui::RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| cx.theme().text);
        svg()
            .path(self.name.path())
            .size(px(self.size.px()))
            .text_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_path_is_well_formed() {
        let path = IconName::Terminal.path();
        assert_eq!(path.as_ref(), "icons/terminal.svg");
    }

    #[test]
    fn every_icon_resolves_through_asset_source() {
        // Each enum variant's path must be loadable from the embedded set,
        // catching a variant whose file is missing from the `icon_assets!` list.
        let assets = KitAssets;
        for name in [
            IconName::Archive,
            IconName::ArrowLeft,
            IconName::ArrowRight,
            IconName::Bot,
            IconName::CalendarClock,
            IconName::Check,
            IconName::ChevronDown,
            IconName::ChevronRight,
            IconName::CircleDot,
            IconName::Ellipsis,
            IconName::FileDiff,
            IconName::FileText,
            IconName::Folder,
            IconName::FolderPlus,
            IconName::Funnel,
            IconName::GitBranch,
            IconName::LayoutGrid,
            IconName::List,
            IconName::ListChecks,
            IconName::ListFilter,
            IconName::MessageSquareText,
            IconName::Minus,
            IconName::PanelLeft,
            IconName::Pencil,
            IconName::Play,
            IconName::Plus,
            IconName::RefreshCw,
            IconName::RotateCcw,
            IconName::Search,
            IconName::Settings,
            IconName::SlidersHorizontal,
            IconName::Terminal,
            IconName::Trash2,
            IconName::X,
            IconName::Zap,
        ] {
            assert!(
                assets
                    .load(name.path().as_ref())
                    .is_ok_and(|loaded| loaded.is_some()),
                "missing embedded svg for {name:?}"
            );
        }
    }

    #[test]
    fn icon_label_is_human_readable() {
        assert_eq!(IconName::Ellipsis.label(), "More actions");
        assert_eq!(IconName::PanelLeft.label(), "Toggle panel");
    }
}
