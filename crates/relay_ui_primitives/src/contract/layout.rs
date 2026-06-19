#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderRule {
    None,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowRule {
    None,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayLayer {
    Floating,
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

pub const BORDER_WIDTH: f32 = 1.0;

pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;

pub const SCROLL_GUTTER_WIDTH: f32 = 10.0;
pub const SCROLL_THUMB_WIDTH: f32 = 5.0;
pub const SCROLL_MIN_THUMB_HEIGHT: f32 = 24.0;

pub const OVERLAY_WINDOW_MARGIN: f32 = 8.0;
pub const OVERLAY_PRIORITY_FLOATING: usize = 1;
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
