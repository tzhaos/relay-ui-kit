use relay_uikit::{
    Tone,
    patterns::{OutputLine, OutputLineStyle, TaskRowData},
};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WorkbenchTask {
    pub id: u64,
    pub title: String,
    pub status_label: &'static str,
    pub branch: &'static str,
    pub worktree: &'static str,
    pub changed: usize,
    pub review: usize,
    pub tone: Tone,
}

impl WorkbenchTask {
    pub fn row_data(&self) -> TaskRowData {
        TaskRowData {
            title: self.title.clone(),
            status_label: self.status_label.to_string(),
            status_tone: self.tone,
            branch: Some(self.branch.to_string()),
            changed: self.changed,
            review: self.review,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WorkbenchSession {
    pub id: u64,
    pub label: String,
    pub detail: String,
    pub tone: Tone,
    pub connected: bool,
}

impl WorkbenchSession {
    fn new(
        id: u64,
        label: impl Into<String>,
        detail: impl Into<String>,
        tone: Tone,
        connected: bool,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            detail: detail.into(),
            tone,
            connected,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WorkbenchReviewReport {
    pub headline: String,
    pub detail: String,
    pub notes: usize,
    pub tone: Tone,
}

impl WorkbenchReviewReport {
    fn new(
        headline: impl Into<String>,
        detail: impl Into<String>,
        notes: usize,
        tone: Tone,
    ) -> Self {
        Self {
            headline: headline.into(),
            detail: detail.into(),
            notes,
            tone,
        }
    }
}

pub(super) fn initial_tasks() -> Vec<WorkbenchTask> {
    vec![
        WorkbenchTask {
            id: 1,
            title: "Implement Relay workbench state".to_string(),
            status_label: "ACTIVE",
            branch: "relay/workbench",
            worktree: "relay-ui-kit",
            changed: 8,
            review: 2,
            tone: Tone::Accent,
        },
        WorkbenchTask {
            id: 2,
            title: "Audit GPUI entity boundaries".to_string(),
            status_label: "REVIEW",
            branch: "relay/gpui-runtime",
            worktree: "crates/relay",
            changed: 4,
            review: 5,
            tone: Tone::Warning,
        },
        WorkbenchTask {
            id: 3,
            title: "Prepare UIKit adapter notes".to_string(),
            status_label: "READY",
            branch: "relay/uikit-adapters",
            worktree: "crates/relay_uikit",
            changed: 2,
            review: 0,
            tone: Tone::Info,
        },
    ]
}

pub(super) fn initial_sessions() -> Vec<WorkbenchSession> {
    vec![
        WorkbenchSession::new(
            11,
            "cargo check",
            "relay-ui-kit / master",
            Tone::Accent,
            true,
        ),
        WorkbenchSession::new(
            12,
            "gallery dogfood",
            "relay_gallery / workbench",
            Tone::Info,
            true,
        ),
        WorkbenchSession::new(
            13,
            "proposal notes",
            "docs / adaptation plan",
            Tone::Muted,
            false,
        ),
    ]
}

pub(super) fn initial_review_report() -> WorkbenchReviewReport {
    WorkbenchReviewReport::new(
        "Review summary ready",
        "Static starter report for the active task.",
        2,
        Tone::Info,
    )
}

pub(super) fn review_report_for_task(task: Option<&WorkbenchTask>) -> WorkbenchReviewReport {
    let Some(task) = task else {
        return WorkbenchReviewReport::new(
            "No task selected",
            "Select a task before refreshing review diagnostics.",
            0,
            Tone::Muted,
        );
    };

    let notes = task.review.max(1);
    WorkbenchReviewReport::new(
        format!("{} review diagnostics", task.status_label),
        format!("{} files changed on {}", task.changed, task.branch),
        notes,
        task.tone,
    )
}

pub(super) fn selected_task(
    tasks: &[WorkbenchTask],
    selected: Option<u64>,
) -> Option<&WorkbenchTask> {
    selected
        .and_then(|id| tasks.iter().find(|task| task.id == id))
        .or_else(|| tasks.first())
}

pub(super) fn selected_session(
    sessions: &[WorkbenchSession],
    selected: Option<u64>,
) -> Option<&WorkbenchSession> {
    selected
        .and_then(|id| sessions.iter().find(|session| session.id == id))
        .or_else(|| sessions.first())
}

pub(super) fn terminal_lines(
    task: Option<&WorkbenchTask>,
    session: Option<&WorkbenchSession>,
) -> Vec<OutputLine> {
    let mut lines = Vec::new();
    let Some(task) = task else {
        lines.push(OutputLine::new("No active task").style(OutputLineStyle::Muted));
        return lines;
    };

    lines.push(
        OutputLine::new(format!("$ codex work {}", task.branch)).style(OutputLineStyle::Input),
    );
    lines.push(OutputLine::new(format!("task: {}", task.title)));
    lines.push(
        OutputLine::new(format!("worktree: {}", task.worktree)).style(OutputLineStyle::Muted),
    );

    if let Some(session) = session {
        let status = if session.connected {
            "attached"
        } else {
            "disconnected"
        };
        lines.push(OutputLine::new(format!(
            "session: {} ({status})",
            session.label
        )));
    } else {
        lines.push(OutputLine::new("session: none").style(OutputLineStyle::Muted));
    }

    lines.push(OutputLine::new(format!(
        "changes: {} files, review notes: {}",
        task.changed, task.review
    )));
    lines.push(
        OutputLine::new("relay selectors keep row selection keyed by stable ids")
            .style(OutputLineStyle::Success),
    );
    lines
}
