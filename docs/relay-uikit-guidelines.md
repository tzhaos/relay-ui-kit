# Relay UIKit Guidelines

## Mission

`relay_uikit` is the implementation surface where Relay's reactive model is tested against real UI complexity.
It should optimize for correctness, robustness, and coverage before stylistic expansion.

## Primary Standards

### Correctness

Every component and pattern should have predictable behavior for:

- value updates;
- controlled and relay-bound usage;
- focus transitions;
- keyboard input;
- pointer interaction;
- disabled and validation states;
- retained rendering and state restoration.

### Robustness

Patterns should behave well under:

- empty collections;
- stale selection after list mutation;
- async loading, reloading, and failure states;
- rapid repeated input;
- overlay open/close churn;
- branch switching across tabs, panes, and split views.

### Completeness

A serious workbench toolkit needs more than isolated controls.
The current crate structure is directionally correct because it includes:

- core inputs and choices;
- feedback and display elements;
- list and row systems;
- overlays and dialogs;
- pickers and command surfaces;
- output, markdown, diff, and source-oriented viewers;
- shell, pane, toolbar, and split-layout primitives;
- a gallery binary for interactive verification.

## Recommended Engineering Rules

1. Keep host-owned state and relay-bound state equally first-class.
   The crate already supports both styles; future components should not regress into only one mode.
2. Prefer explicit state machines over incidental boolean coupling.
   Overlays, pickers, and input validation paths become fragile quickly.
3. Test keyboard and selection behavior as seriously as visuals.
   Desktop UI quality depends on navigation fidelity, not just rendering.
4. Treat the gallery as a verification tool, not a showcase only.
   Stress scenes and edge-state demos should keep growing with the kit.
5. Keep visual policy separate from state semantics.
   Theme, motion, and tone are design layers; selection/resource/form correctness belongs below them.

## Relationship to Relay

`relay_uikit` should consume Relay primitives, not reimplement them.

Good examples:

- using `Binding<T>` for ordinary two-way controls;
- using `Selector<K>` for keyed list selection;
- using `Resource<T, E>` presentation helpers for async surfaces;
- using `SubView` and `KeyedSubViews` where retained child entities matter.

Less desirable directions:

- inventing a second state system inside UIKit;
- hiding entity lifecycle behind opaque abstractions;
- coupling visual widgets to app-specific domain state.

## What “Complete” Means for This Crate

Completeness here does not mean infinite component count.
It means the crate can support a full desktop/workbench workflow with reliable primitives across:

- settings forms;
- command palettes and pickers;
- hierarchical navigation;
- activity/session/task surfaces;
- file/source/diff/output inspection;
- split-pane multi-panel shells.

That is the bar worth documenting and testing against.
