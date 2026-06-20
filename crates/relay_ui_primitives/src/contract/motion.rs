use std::time::Duration;

/// How long a motion animation takes to complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDuration {
    /// Near-instant animation (50 ms).
    Instant,
    /// A quick animation (150 ms).
    Fast,
    /// A slower, more deliberate animation (300 ms).
    Slow,
}

impl MotionDuration {
    pub fn duration(self) -> Duration {
        match self {
            MotionDuration::Instant => Duration::from_millis(50),
            MotionDuration::Fast => Duration::from_millis(150),
            MotionDuration::Slow => Duration::from_millis(300),
        }
    }
}

impl From<MotionDuration> for Duration {
    fn from(value: MotionDuration) -> Self {
        value.duration()
    }
}

/// The direction a motion animation originates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDirection {
    /// The animation starts from the bottom and moves upward.
    FromBottom,
    /// The animation starts from the left and moves rightward.
    FromLeft,
    /// The animation starts from the right and moves leftward.
    FromRight,
    /// The animation starts from the top and moves downward.
    FromTop,
}

/// What kind of motion a component uses.
///
/// Currently only [`Entry`] and [`ContinuousFeedback`] are in active use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPolicy {
    /// Animate the component when it first appears (slide-in + optional fade).
    Entry,
    /// A repeating animation that runs as long as the component is visible.
    ContinuousFeedback,
}

/// When a component's motion should be disabled.
///
/// Currently only [`HostDisabled`] is used — the host view suppresses animation
/// by omitting the motion extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDisablePolicy {
    /// The host can disable motion (e.g., for tests or reduced-motion mode).
    HostDisabled,
}

/// A motion specification for a single component.
///
/// Each rule declares what animation a component uses, how long it
/// lasts, which direction it moves, and under what conditions the
/// animation may be disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionRule {
    /// The component name (used for lookups).
    pub component: &'static str,
    /// The kind of motion (entry or continuous feedback).
    pub policy: MotionPolicy,
    /// How long the animation takes.
    pub duration: MotionDuration,
    /// The direction the animation originates from, when applicable.
    pub direction: Option<MotionDirection>,
    /// When the motion may be disabled.
    pub disable_policy: MotionDisablePolicy,
}

/// The canonical set of motion rules for all components that use animation.
///
/// Each entry maps a component name to its motion policy, duration,
/// direction, and disable policy.
pub const MOTION_RULES: &[MotionRule] = &[
    MotionRule {
        component: "CommandPalette",
        policy: MotionPolicy::Entry,
        duration: MotionDuration::Fast,
        direction: Some(MotionDirection::FromTop),
        disable_policy: MotionDisablePolicy::HostDisabled,
    },
    MotionRule {
        component: "DropdownMenu",
        policy: MotionPolicy::Entry,
        duration: MotionDuration::Fast,
        direction: Some(MotionDirection::FromTop),
        disable_policy: MotionDisablePolicy::HostDisabled,
    },
    MotionRule {
        component: "LauncherMenu",
        policy: MotionPolicy::Entry,
        duration: MotionDuration::Fast,
        direction: Some(MotionDirection::FromTop),
        disable_policy: MotionDisablePolicy::HostDisabled,
    },
    MotionRule {
        component: "LoadingSpinner",
        policy: MotionPolicy::ContinuousFeedback,
        duration: MotionDuration::Slow,
        direction: None,
        disable_policy: MotionDisablePolicy::HostDisabled,
    },
    MotionRule {
        component: "Skeleton",
        policy: MotionPolicy::ContinuousFeedback,
        duration: MotionDuration::Slow,
        direction: None,
        disable_policy: MotionDisablePolicy::HostDisabled,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_entry_motion_is_fast_from_top() {
        let rule = MOTION_RULES
            .iter()
            .find(|rule| rule.component == "CommandPalette")
            .unwrap();

        assert_eq!(rule.duration, MotionDuration::Fast);
        assert_eq!(rule.direction, Some(MotionDirection::FromTop));
    }

    #[test]
    fn motion_duration_values_are_stable() {
        assert_eq!(MotionDuration::Fast.duration(), Duration::from_millis(150));
    }
}
