use relay::Resource;

use crate::patterns::{OutputLine, OutputLineStyle};

/// Render-ready state for an output log backed by a Relay resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputResourceSnapshot {
    pub lines: Vec<OutputLine>,
    pub loading: bool,
    pub status_text: String,
}

/// Fold a `Resource<Vec<OutputLine>, E>` into the common output-log view shape.
pub fn output_resource_snapshot<E>(
    resource: &Resource<Vec<OutputLine>, E>,
    cx: &gpui::App,
    pending_text: impl Into<String>,
    refreshing_text: impl Into<String>,
    ready_text: impl FnOnce(usize) -> String,
    error_status: impl Into<String>,
    error_line: impl FnOnce(&E) -> String,
) -> OutputResourceSnapshot {
    let pending_text = pending_text.into();
    let refreshing_text = refreshing_text.into();
    let error_status = error_status.into();

    resource.fold_latest(
        cx,
        || OutputResourceSnapshot {
            lines: vec![OutputLine::new(pending_text.clone()).style(OutputLineStyle::Muted)],
            loading: true,
            status_text: pending_text,
        },
        |lines, loading| OutputResourceSnapshot {
            lines: lines.clone(),
            loading,
            status_text: if loading {
                refreshing_text
            } else {
                ready_text(lines.len())
            },
        },
        |error| OutputResourceSnapshot {
            lines: vec![OutputLine::new(error_line(error)).style(OutputLineStyle::Error)],
            loading: false,
            status_text: error_status,
        },
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::TestApp;

    use super::*;

    #[test]
    fn output_resource_snapshot_retains_latest_lines_while_reloading() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            relay::init(cx);
            Resource::<Vec<OutputLine>, String>::ready(cx, vec![OutputLine::new("ready")])
        });

        app.update(|cx| {
            resource.reload(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(vec![OutputLine::new("new")])
            });
        });

        let snapshot = app.read(|cx| {
            output_resource_snapshot(
                &resource,
                cx,
                "Loading output",
                "Refreshing output",
                |line_count| format!("{line_count} lines ready"),
                "Refresh failed",
                |error| format!("refresh failed: {error}"),
            )
        });

        assert_eq!(
            snapshot.lines.first().map(|line| line.text.as_str()),
            Some("ready")
        );
        assert!(snapshot.loading);
        assert_eq!(snapshot.status_text, "Refreshing output");
    }
}
