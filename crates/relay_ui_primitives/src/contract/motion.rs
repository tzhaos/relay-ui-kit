use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDuration {
    Instant,
    Fast,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDirection {
    FromBottom,
    FromLeft,
    FromRight,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionRule {
    pub component: &'static str,
    pub policy: MotionPolicy,
    pub duration: MotionDuration,
    pub direction: Option<MotionDirection>,
    pub disable_policy: MotionDisablePolicy,
}

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
