# Relay Reinforcement Opportunities From Web Architecture

## Purpose

This note does not ask whether Relay should copy React, Vue, or SolidJS.
Instead, it asks a narrower and more useful question:

Given the actual architecture of GPUI and Zed, where would selected web-framework ideas materially improve developer experience?

The examples below come from concrete Zed surfaces:

- project panel
- picker
- command palette
- terminal view
- notifications

## Opportunity 1: Derived state is still largely manual and structural

### Evidence

The project panel maintains a large, hand-derived state structure:

- `State.visible_entries`, `ancestors`, `max_width_item_index`, `edit_state`, `expanded_dir_ids`, etc. in [project_panel.rs](F:/Workspace/zed/crates/project_panel/src/project_panel.rs:107)
- recomputation is orchestrated manually in `update_visible_entries(...)` in [project_panel.rs](F:/Workspace/zed/crates/project_panel/src/project_panel.rs:4041)

This method:

- snapshots inputs from project/settings/git state;
- spawns background work;
- rebuilds a new derived structure;
- re-applies selection/focus/autoscroll side effects on completion.

### What this suggests

This is the classic place where SolidJS-style derivation and Vue-style composables help:

- source signals for project snapshot, fold state, selection, filter settings;
- memoized derived tree/list projections;
- explicit resource/query state for background recomputation;
- effect boundaries for focus/autoscroll follow-up.

### What Relay should offer

- `QueryResource<T>` or `ComputedResource<T>` for async derived projections
- `SelectionModel<K>` and `TreeExpansionModel<K>`
- `DerivedVec<T>` / `FilteredVec<T>` patterns for large UI projections
- composables such as `use_tree_projection(...)`

The goal is not to remove background work.
The goal is to make the projection graph explicit instead of hand-wired.

## Opportunity 2: Query/update state machines are handwritten repeatedly

### Evidence

`Picker` contains a reusable but still manual async-update protocol:

- `PendingUpdateMatches` in [picker.rs](F:/Workspace/zed/crates/picker/src/picker.rs:61)
- `pending_update_matches` and `confirm_on_update` in [picker.rs](F:/Workspace/zed/crates/picker/src/picker.rs:70)
- `update_matches_with_options(...)` in [picker.rs](F:/Workspace/zed/crates/picker/src/picker.rs:695)
- follow-up completion handling in [picker.rs](F:/Workspace/zed/crates/picker/src/picker.rs:715)

The command palette delegate then implements another layer of the same problem:

- `latest_query`, `selected_ix`, `updating_matches`, `query_history` in [command_palette.rs](F:/Workspace/zed/crates/command_palette/src/command_palette.rs:159)
- query-to-background-match pipeline in [command_palette.rs](F:/Workspace/zed/crates/command_palette/src/command_palette.rs:436)
- synchronous “finalize if possible” optimization in [command_palette.rs](F:/Workspace/zed/crates/command_palette/src/command_palette.rs:535)

### What this suggests

This is a good target for web-inspired async query abstractions:

- Solid-style resources;
- Vue-query / TanStack-query style request lifecycle;
- explicit stale/pending/ready semantics;
- cancellation-aware background completion.

### What Relay should offer

- `use_async_query(query_key, load_fn)`
- `use_ranked_query(query, rank_fn)`
- `ResourceState::{Idle, Pending, Reloading, Ready, Error}`
- built-in stale-result suppression keyed by generation
- optional “block briefly for warm results” policy hook

This would let picker-like surfaces declare:

- current query
- selected item
- async candidate list
- confirm-after-update policy

without each delegate rebuilding the control flow manually.

## Opportunity 3: Selection logic is frequently central but remains ad hoc

### Evidence

The command palette stores:

- `selected_ix` in [command_palette.rs](F:/Workspace/zed/crates/command_palette/src/command_palette.rs:165)

The picker stores:

- delegate-owned `selected_index`
- navigation helpers in [picker.rs](F:/Workspace/zed/crates/picker/src/picker.rs:422)

The project panel stores:

- `selection`
- `marked_entries`
- sticky selection and visible-entry indexing logic across many methods in [project_panel.rs](F:/Workspace/zed/crates/project_panel/src/project_panel.rs:1249) and throughout `update_visible_entries`.

### What this suggests

Relay already has a useful `Selector<K>`, but the evidence suggests it should grow into a broader family of selection composables:

- single selection
- marked/multi selection
- projected selected item
- selection with history
- selection with reveal/scroll policy

### What Relay should offer

- `use_single_selection<K>()`
- `use_multi_selection<K>()`
- `use_selection_projection(items, key_fn)`
- `use_keyboard_selection(...)`
- `use_scroll_synced_selection(...)`

This is a place where SolidJS-like fine-grained selection signals can provide real value, especially for large retained lists.

## Opportunity 4: Focus and lifecycle coordination is explicit, repetitive, and important

### Evidence

Terminal view setup includes:

- focus subscriptions in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:243)
- global settings observation in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:273)
- terminal event subscriptions in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:1080)

Pane setup includes:

- focus hooks, settings observation, project subscription in [pane.rs](F:/Workspace/zed/crates/workspace/src/pane.rs:544)

### What this suggests

React-style hooks are not the right shape here, but Vue-style composables absolutely are.
These are not UI rendering problems.
They are entity-lifetime coordination problems.

### What Relay should offer

- `use_focus_state(...)`
- `use_entity_subscription(...)`
- `use_window_event(...)`
- `use_settings_observer(...)`
- `use_release_cleanup(...)`

These should compile down to GPUI-native subscriptions, but package the repetitive lifecycle code better.

## Opportunity 5: Async follow-up side effects are mixed into view/controller code

### Evidence

Project panel:

- background recompute + apply + focus/autoscroll in [project_panel.rs](F:/Workspace/zed/crates/project_panel/src/project_panel.rs:4075)

Terminal view:

- hover target async updates and task replacement in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:1114)
- cloning into a new entity through async work in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:1724)

Notifications:

- deferred app-wide notification propagation in [notifications.rs](F:/Workspace/zed/crates/workspace/src/notifications.rs:1455)

### What this suggests

Many surfaces need a clean split between:

- source state
- background work
- UI-visible resource state
- side effects after resource completion

This maps very naturally to:

- resource primitives
- `watch` / `watch_changes`
- effect-with-cleanup
- query/mutation composables

### What Relay should offer

- `use_query(...)`
- `use_mutation(...)`
- `use_async_projection(...)`
- `use_followup_effect(...)`
- `use_optimistic_state(...)`

## Opportunity 6: Data-heavy surfaces need stronger view-model composition, not more macros

### Evidence

Command palette logic is split across:

- command collection
- fuzzy matching
- interception
- query history
- selection
- dispatch

in one delegate type, see [command_palette.rs](F:/Workspace/zed/crates/command_palette/src/command_palette.rs:147).

Project panel similarly combines:

- tree projection
- edit mode
- selection
- drag hover
- diagnostics
- filtering/sorting

inside one large surface state type, see [project_panel.rs](F:/Workspace/zed/crates/project_panel/src/project_panel.rs:135).

### What this suggests

The biggest DX gap is not syntax.
It is absence of a view-model composition layer.

### What Relay should offer

Composable state modules such as:

- `CommandPaletteModel`
- `PickerModel`
- `TreePanelModel`
- `NotificationModel`
- `TerminalSearchModel`

These can be implemented as strongly typed Rust structs built from signals, memos, resources, selectors, and entity-scoped effects.

This is closer to Vue composables and modern app-model architecture than to classic React hooks.

## Opportunity 7: Render-time code still contains some stateful imperative seams that want better boundaries

### Evidence

Terminal view render includes imperative synchronization:

- `scroll_handle.update(...)`
- conditional display offset application

inside `render(...)` in [terminal_view.rs](F:/Workspace/zed/crates/terminal_view/src/terminal_view.rs:1289)

This is explicitly marked by a `TODO`.

### What this suggests

Some state transitions currently live “just before drawing” because there isn't a cleaner declarative coordination layer.

### What Relay should offer

- effect scopes aligned to entity lifecycle
- render-safe derived state staging
- explicit “before render / after source change / after async completion” helpers

This is where SolidJS-style effect discipline can improve architecture without altering GPUI's rendering ownership.

## Summary Matrix

### Best ideas to import from SolidJS

- signals for fine-grained state
- memos for derived projections
- explicit `untrack`
- effect/resource semantics

### Best ideas to import from Vue

- composables as reusable stateful logic units
- strong separation between primitive layer and app-model layer
- declarative async/query conventions

### Best ideas to import from React

- understandable component/state boundaries
- reusable stateful patterns with predictable mental models
- emphasis on devtools/testable state flow

### Ideas not worth importing directly

- browser reconciliation assumptions
- universal hook-order runtime rules
- proxy magic
- trying to make entities disappear

## Final Take

The evidence suggests that Relay's biggest contribution should be:

- not another renderer;
- not another event bus;
- not a macro-heavy imitation of web components;

but a GPUI-native state/composable layer that:

- makes derived state explicit;
- makes async query/mutation flows standard;
- packages focus/subscription/lifecycle coordination cleanly;
- keeps performance escape hatches visible;
- and lets desktop workbench surfaces be built from reusable view-model modules instead of giant hand-wired controllers.
