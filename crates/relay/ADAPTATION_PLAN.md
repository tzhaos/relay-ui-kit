# Relay Adaptation Plan

Relay is GPUI's reactive state/runtime layer. Its app-facing API should make
SolidJS-like state flow convenient while keeping GPUI's entity, element, and
window lifecycles as the source of truth.

For the current completion audit and migration checklist, see
[MIGRATION_PLAN.md](MIGRATION_PLAN.md).

## Current Landed Surface

- `Signal`, `Binding`, `Memo` / `derived`, `Effect`, `watch`, `untrack`, and
  reactive context cover app state, derivation, side effects, and cross-layer
  state sharing.
- `effect_with_cleanup` and `effect_in_with_cleanup` cover source-dependent
  side-effect lifetimes. Cleanups run before the next effect body and on
  dispose/entity release; cleanup reads are untracked while cleanup writes still
  notify normally. This gives the useful SolidJS-style cleanup behavior without
  introducing a separate owner tree outside GPUI's entity lifecycle.
- `StateScope` is the view-field lifetime holder for entity-scoped effects,
  source-driven resource watchers, and dirty-check forms. It keeps handles
  reachable with the owning GPUI entity; entity-scoped cleanup still comes from
  GPUI `on_release`, while app-scoped effects should use explicit `Effect`
  handles when manual disposal is needed.
- `watch` now tracks only the declared `sources`; its `react` closure runs in
  `untrack` so side-effect reads do not expand the source set. `watch_changes`
  covers the common "seed initial state, react only to later source changes"
  case.
- `StateScope::reload_resource_from_source` and
  `StateScope::load_resource_from_source` package the repeated source snapshot
  resource shape: track source reads once, pass the same snapshot into the load
  builder, and keep the resource UI-agnostic. The owning GPUI entity still owns
  the source tracking lifecycle.
- `StateScope::reload_resource_on_changes` and
  `StateScope::load_resource_on_changes` remain available for the more flexible
  shape where source declaration and load construction intentionally differ.
- `StateScope::form()` packages dirty-check-only form lifetime into the owning
  GPUI entity. Views that need reset or commit keep a `Form` field directly;
  views that only need `is_dirty` can avoid `std::mem::forget(form)`.
- `SignalVecExt` covers common `Signal<Vec<T>>` structural mutations,
  including `extend` for appending multiple items with a single reactive
  notification.
- `SignalVecExt::push_selected_by` covers the repeated retained-row creation
  shape: append the host-owned item and select its stable key in one batch.
- `SignalVecExt::remove_selected_by` covers the repeated selector-backed
  removal shape in retained row hosts. It removes the currently selected item
  from a `Signal<Vec<T>>` and reconciles the `Selector<K>` inside one batch,
  without introducing a collection store or moving item ownership into Relay.
- `Resource::load`, `Resource::reload`, status query helpers, `latest`,
  `read_error`, and `fold_latest` cover async pending, reloading, ready, and
  error states without baking in a UI boundary.
- `SubView` and `KeyedSubViews` expose GPUI entity-grained UI retention.
  `KeyedSubViews::sync_with_selector` covers the repeated selected-retained-row
  shape: the host owns item order, `Selector<K>` owns selection, and row entity
  retention stays in `KeyedSubViews` while stale selected keys are reconciled
  before sync.
- `Selector<K>` gives selection-heavy lists per-key tracking, can reconcile
  selection against the current item keys, and owns ordered next/previous plus
  first/last navigation for list and command surfaces. Its `_by` helpers let
  hosts navigate or reconcile item collections by stable key without cloning
  the collection just to build key iterators.
- `SelectedItemExt` covers the repeated selector-backed projection from
  `Signal<Vec<T>>` or `Memo<Vec<T>>` to `Memo<Option<T>>`. Use
  `selected_by` for exact selected-key projection and `selected_by_or_first`
  when the app wants first-item fallback without mutating the selector.
- `#[derive(Reactive)]` supports nested reactive state wrappers for field-level
  app state.
- `relay_uikit` has begun consuming `Selector<K>` for task/session/tab
  selection, including keyed gallery hosts where selection is host-owned rather
  than embedded in row data.
- The gallery Patterns output surface consumes `Resource::reload` / `latest`
  semantics through `fold_latest`, keeping the previous output visible while an
  async refresh is in flight.
- Resource status reads (`is_loading`, `has_latest`, `read_error`,
  `error_value`) are available at the runtime layer, so app surfaces can expose
  loading/error state without cloning or matching the whole resource state.
- The gallery Patterns item picker now has an app-like keyed host that combines
  `KeyedSubViews` row retention with `Selector<u64>` mutual selection, with
  tests covering row entity reuse across reorder and selected-row updates. The
  picker uses selector reconciliation so removed keys cannot leave stale
  selection behind.
- The gallery Stress session list uses the same `KeyedSubViews` + `Selector`
  shape for dynamic session rows, with tests covering row reuse during
  selection changes and selection reconciliation after removing the active row.
- Existing keyed hosts use `Selector::select_next` for cycling active rows, so
  ordered selection behavior lives in Relay instead of being reimplemented by
  each app host.
- Keyed hosts now use `Selector` `_by` helpers where they own item structs,
  keeping key extraction close to the host collection and avoiding untracked
  whole-list clones for navigation.
- The compiled `keyed_subviews` Relay example now pairs `KeyedSubViews` with
  `Selector<u64>` and host-level arrow-key handling. Its tests verify row
  entity reuse while selection moves, previous-selection wraparound, and Enter
  acting on the selected row. It now uses `sync_with_selector` so selection
  reconciliation and row entity sync share one key projection.
- The compiled `session_surface` Relay example promotes that shape outside
  gallery into a GPUI session surface. It combines dynamic `Signal<Vec<T>>`
  sessions, retained row entities, per-row local state, host-level
  Home/End/arrow navigation, Enter activation, and Delete removal. Its tests
  verify row entity reuse, row-local state survival across reorder, and command
  keyboard behavior, with selected row sync going through
  `KeyedSubViews::sync_with_selector`.
- The compiled `branch_subviews` Relay example promotes persistent branch state
  outside UIKit. It keeps each branch as a host-owned `SubView`, renders only
  the active branch, and tests that inactive branches do not render while their
  GPUI entity state survives switching away and back.
- The compiled `source_resource` Relay example promotes source-driven resource
  loading outside gallery/workbench. It uses `StateScope::load_resource_from_source`
  to keep `Resource` UI-agnostic, scope source tracking to the owning GPUI
  entity, start from `Pending`, and test that source changes later enter
  `Reloading` while retaining the last ready value.
- The compiled `effect_cleanup` Relay example promotes source-dependent
  side-effect lifetime outside UIKit. It switches a channel subscription from a
  signal source and verifies the old cleanup runs before the new subscription
  is installed.
- `StateScope::effect_in_with_cleanup` is covered by a focused release test:
  storing only the `StateScope` field is enough for the entity-scoped effect to
  clean up on GPUI entity release, because the cleanup path is registered with
  `cx.on_release`.
- The compiled `command_picker` Relay example promotes command/picker-shaped
  host state outside UIKit. It combines host-owned command data, query
  `Binding`, filtered `Memo`, and `Selector<&'static str>` navigation/execution
  without introducing a Relay command registry.
- The same `command_picker` example now derives `selected_command` with an
  ordinary `SelectedItemExt` projection from filtered commands plus
  `Selector`. This keeps command/picker data host-owned while removing the
  repeated selected-item memo closure.
- The gallery workbench page is now a compiled app-like surface. It wires
  task/session state through stable-id `Selector<u64>` values, renders the task
  rail and session context list as `KeyedSubViews`, and keeps center/status
  projections reading from the same runtime state. Its selected retained-row
  hosts use `sync_with_selector`; tests cover task row entity reuse while
  selection changes and session selection cleanup when the active session is
  removed.
- The workbench selected task/session projections are derived with `Memo` from
  `Signal<Vec<T>>` plus `Selector<u64>` through `SelectedItemExt`. Render code
  reads the selected item memo instead of cloning whole lists in each pane, and
  the projection helper now lives in Relay because this exact shape repeated
  across Workbench and command/picker surfaces.
- `relay_uikit` command and picker rows now accept selector-backed selection
  where the component key model is already static: `CommandRow::selected_by`
  reuses `SelectionBinding`, while `ItemPicker::selected_by` reads and writes a
  `Selector<&'static str>`. This keeps component APIs value-first while removing
  repeated host glue for command/picker selection.
- The gallery Patterns surface now exercises those selector-backed command and
  picker adapters in a compiled app-shaped path. Command rows and the branch
  picker share host-owned `Selector<&'static str>` state, with tests covering
  the selected command/branch labels after selector changes.
- The workbench Review panel now owns a `Resource<WorkbenchReviewReport, String>`
  and renders it through `fold_latest`. It is source-driven from the selected
  task, so task changes automatically refresh diagnostics while the last ready
  report remains visible.
- The workbench center transcript now owns a
  `Resource<Vec<OutputLine>, String>`. Its refresh action snapshots the
  currently selected task/session, enters `Reloading`, and keeps the previous
  terminal output visible until the new transcript resolves.
- The workbench transcript and review report now use
  `StateScope::reload_resource_from_source`: selected task/session memos are
  tracked once as source snapshots, then handed to the async load builder. Task
  or session changes automatically reload while retaining the last ready output.
- `relay_uikit` now exposes `output_resource_snapshot` for the repeated
  output-log resource shape. This is deliberately narrower than a generic async
  boundary: it only folds `Resource<Vec<OutputLine>, E>` into
  lines/loading/status for `OutputSurface` + `OutputLog` call sites.
- `Resource::fold_latest` docs and tests cover borrowed, non-`Clone` latest
  values, reinforcing that Relay provides resource state semantics while
  component crates own any render-ready adapter shape.

## List Boundary

Use two different list paths:

- Lightweight element lists: map items to GPUI elements directly, or use the
  UIKit `ForEach` helper when the source is a `Signal<Vec<T>>`. This subscribes
  the surrounding view and rebuilds child elements during parent render.
- Stateful or heavy row lists: store `KeyedSubViews<K, RowView>` in the host
  entity. Each row gets a stable GPUI entity keyed by `K`, and clean siblings
  can reuse GPUI's view cache.

Choose `KeyedSubViews` when a row owns state, focus or scroll-like element
state, scoped effects, async resources, or enough rendering work that sibling
rows should remain cached. Otherwise prefer the simpler element path.

## Branch Boundary

Persistent conditional UI should use the same GPUI entity boundary as retained
rows. For tabs, panes, and view modes that own state, store each branch as a
`SubView<T>` field in the host entity and render the active branch through
`cached(...)`.

This is the useful GPUI-shaped subset of Show/Switch-style design. GPUI's
`hidden()` sets `display: none`; hidden children are skipped during layout,
prepaint, and paint, so it should not be treated as a branch lifecycle
primitive. Add a Relay branch helper only after repeated app code shows a
common typed shape beyond ordinary host-owned `SubView` fields.

## UIKit Adaptation

Keep presentation components value-first and `Binding`-friendly. Add Relay
runtime adapters only where they simplify real app state:

- Use `Binding<T>` for ordinary two-way form controls.
- Use `Selector<K>` for mutually exclusive row, tab, picker, and command
  selection.
- Use `SelectedItemExt` when a host needs the selected item memo from a
  selector-backed `Signal<Vec<T>>` or filtered `Memo<Vec<T>>`.
- Use `KeyedSubViews` in host entities for stateful or heavy repeated rows. Use
  `sync_with_selector` when those retained rows are backed by a `Selector<K>`,
  so selector reconciliation and row entity sync cannot drift apart.
- Keep `ForEach` focused on lightweight element lists; row entity caching lives
  in host entities through `KeyedSubViews`.
- Use `Resource::reload` / `latest` for async data surfaces that should keep
  stale-ready content visible while refreshing. For terminal/output-log
  resources, use `output_resource_snapshot`; for other data shapes, keep local
  `fold_latest` render branches until repeated real app surfaces justify a
  similarly narrow helper.
- Use `StateScope::load_resource_from_source` for source-driven resources that
  start pending, and `StateScope::reload_resource_from_source` when the initial
  value is already ready. Use the `_on_changes` variants only when tracked
  sources and load construction intentionally differ. This gives the useful
  part of SolidJS-style source resources while keeping the resource itself
  UI-agnostic and scoped to the owning GPUI entity.
- Use `batch` or single-operation collection helpers such as
  `SignalVecExt::extend`, `SignalVecExt::push_selected_by`, and
  `SignalVecExt::remove_selected_by` for write bursts that should notify once.
  Avoid assuming a global frame-boundary batch when code does not have a
  `Window` lifecycle hook.
- Use `effect_in_with_cleanup` when a source-dependent effect owns a temporary
  external handle. Keep the source reads inside the effect body, register the
  handle release with `CleanupScope::on_cleanup`, and let GPUI entity release
  dispose the effect.

## Next Landing Steps

1. Do not add a `Resource::from_source` / `create_resource` constructor yet.
   The Workbench transcript/review report and Relay `source_resource` example
   fit the smaller `StateScope::*resource_from_source` helpers, which match
   GPUI entity lifetime better and keep `Resource` independent from source
   tracking.
2. Do not add a Show/Switch helper yet. The `branch_subviews` example covers
   persistent branch state with GPUI entity boundaries; revisit only if repeated
   app code shows a common typed helper would remove real boilerplate.
3. Do not add `watch_with_cleanup` yet. The lower-level
   `effect_in_with_cleanup` covers the real subscription/listener lifetime
   case, while existing `watch` remains the simpler source/react split for
   ordinary side effects. Keep `StateScope` usage entity-scoped; app-scoped
   effects should keep explicit `Effect` handles. Add a watch-specific cleanup
   helper only after repeated app code shows the source/react split plus cleanup
   is common enough to justify another API.
4. Do not add a Relay command registry or selector-backed collection store yet.
   `SelectedItemExt` covers the repeated selected-item projection without
   taking ownership of command data, item ordering, row rendering, or UIKit
   presentation. `SignalVecExt::push_selected_by` and
   `SignalVecExt::remove_selected_by` are deliberately narrower: they only
   batch append-select and selected-item removal/reconciliation for host-owned
   vectors.
5. Keep expanding compiled app-shaped surfaces before adding broader Relay
   primitives. The workbench migration did not require a new UIKit adapter:
   existing `selected_by` / `active_by` hooks were enough once state lived in
   `Selector<K>` and row retention lived in host-owned `KeyedSubViews`.

## Verification Gates

- `cargo test -p relay`
- `cargo test -p relay --examples`
- `cargo check -p relay --examples`
- `cargo test -p relay_uikit`
- `cargo check -p relay_uikit --bin relay_gallery`
- `cargo check --workspace`
- `git diff --check`
