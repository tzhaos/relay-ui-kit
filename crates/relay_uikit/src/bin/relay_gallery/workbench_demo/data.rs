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
    fn new(
        id: u64,
        title: impl Into<String>,
        status_label: &'static str,
        branch: &'static str,
        worktree: &'static str,
        changed: usize,
        review: usize,
        tone: Tone,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            status_label,
            branch,
            worktree,
            changed,
            review,
            tone,
        }
    }

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

pub(super) fn initial_tasks() -> Vec<WorkbenchTask> {
    vec![
        WorkbenchTask::new(
            1,
            "Implement Relay workbench state",
            "ACTIVE",
            "relay/workbench",
            "relay-ui-kit",
            8,
            2,
            Tone::Accent,
        ),
        WorkbenchTask::new(
            2,
            "Audit GPUI entity boundaries",
            "REVIEW",
            "relay/gpui-runtime",
            "crates/relay",
            4,
            5,
            Tone::Warning,
        ),
        WorkbenchTask::new(
            3,
            "Prepare UIKit adapter notes",
            "READY",
            "relay/uikit-adapters",
            "crates/relay_uikit",
            2,
            0,
            Tone::Info,
        ),
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
