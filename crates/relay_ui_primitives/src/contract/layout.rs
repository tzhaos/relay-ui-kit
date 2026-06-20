/// Whether an element draws a border and at what thickness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderRule {
    /// No border is drawn.
    None,
    /// A 1 px hairline border is drawn.
    Hairline,
}

impl BorderRule {
    pub fn width(self) -> f32 {
        match self {
            BorderRule::None => 0.0,
            BorderRule::Hairline => BORDER_WIDTH,
        }
    }
}

/// The shadow style applied to an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowRule {
    /// No shadow is drawn.
    None,
    /// An overlay shadow for floating elements (menus, dialogs, etc.).
    Overlay,
}

/// The z-ordering layer for an overlay element.
///
/// Higher-priority layers paint on top of lower-priority ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayLayer {
    /// A floating overlay such as a menu or popover.
    Floating,
    /// A modal dialog that blocks interaction with layers beneath it.
    Dialog,
}

impl OverlayLayer {
    pub fn priority(self) -> usize {
        match self {
            OverlayLayer::Floating => OVERLAY_PRIORITY_FLOATING,
            OverlayLayer::Dialog => OVERLAY_PRIORITY_DIALOG,
        }
    }
}

/// Standard hairline border width in device pixels.
pub const BORDER_WIDTH: f32 = 1.0;

/// Small corner radius (4 px).
pub const RADIUS_SM: f32 = 4.0;
/// Medium corner radius (6 px).
pub const RADIUS_MD: f32 = 6.0;
/// Large corner radius (8 px).
pub const RADIUS_LG: f32 = 8.0;

/// Width of the scrollbar gutter area (the track).
pub const SCROLL_GUTTER_WIDTH: f32 = 10.0;
/// Width of the scrollbar thumb.
pub const SCROLL_THUMB_WIDTH: f32 = 5.0;
/// Minimum height of the scrollbar thumb.
pub const SCROLL_MIN_THUMB_HEIGHT: f32 = 24.0;

/// Margin between overlay windows and the viewport edge.
pub const OVERLAY_WINDOW_MARGIN: f32 = 8.0;
/// Z-priority for floating overlays (menus, popovers).
pub const OVERLAY_PRIORITY_FLOATING: usize = 1;
/// Z-priority for modal dialogs (paints above floating overlays).
pub const OVERLAY_PRIORITY_DIALOG: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_rule_uses_one_pixel_hairlines() {
        assert_eq!(BorderRule::Hairline.width(), 1.0);
    }

    #[test]
    fn dialog_overlay_sits_above_floating_overlay() {
        assert!(OverlayLayer::Dialog.priority() > OverlayLayer::Floating.priority());
    }

    #[test]
    fn default_radii_stay_in_small_desktop_range() {
        assert_eq!(RADIUS_LG, 8.0);
    }
}
