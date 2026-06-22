# Relay Adaptation Plan

Relay is GPUI's reactive state/runtime layer. Its app-facing API should make
SolidJS-like state flow convenient while keeping GPUI's entity, element, and
window lifecycles as the source of truth.

## Current Landed Surface

- `Signal`, `Binding`, `Memo` / `derived`, `Effect`, `watch`, `untrack`, and
  reactive context cover app state, derivation, side effects, and cross-layer
  state sharing.
- `SignalVecExt` covers common `Signal<Vec<T>>` structural mutations,
  including `extend` for appending multiple items with a single reactive
  notification.
- `Resource::load`, `Resource::reload`, status query helpers, `latest`,
  `read_error`, and `fold_latest` cover async pending, reloading, ready, and
  error states without baking in a UI boundary.
- `SubView` and `KeyedSubViews` expose GPUI entity-grained UI retention.
- `Selector<K>` gives selection-heavy lists per-key tracking, can reconcile
  selection against the current item keys, and owns ordered next/previous plus
  first/last navigation for list and command surfaces. Its `_by` helpers let
  hosts navigate or reconcile item collections by stable key without cloning
  the collection just to build key iterators.
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
  acting on the selected row.
- The compiled `session_surface` Relay example promotes that shape outside
  gallery into a GPUI session surface. It combines dynamic `Signal<Vec<T>>`
  sessions, retained row entities, per-row local state, host-level
  Home/End/arrow navigation, Enter activation, and Delete removal. Its tests
  verify row entity reuse, row-local state survival across reorder, and command
  keyboard behavior.
- The gallery workbench page is now a compiled app-like surface. It wires
  task/session state through stable-id `Selector<u64>` values, renders the task
  rail and session context list as `KeyedSubViews`, and keeps center/status
  projections reading from the same runtime state. Its tests cover task row
  entity reuse while selection changes and session selection cleanup when the
  active session is removed.
- The workbench selected task/session projections are derived with `Memo` from
  `Signal<Vec<T>>` plus `Selector<u64>`. Render code reads the selected item
  memo instead of cloning whole lists in each pane, showing that existing Relay
  derived state covers this app pattern without a selector-specific item helper.
- `relay_uikit` command and picker rows now accept selector-backed selection
  where the component key model is already static: `CommandRow::selected_by`
  reuses `SelectionBinding`, while `ItemPicker::selected_by` reads and writes a
  `Selector<&'static str>`. This keeps component APIs value-first while removing
  repeated host glue for command/picker selection.

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

## UIKit Adaptation

Keep presentation components value-first and `Binding`-friendly. Add Relay
runtime adapters only where they simplify real app state:

- Use `Binding<T>` for ordinary two-way form controls.
- Use `Selector<K>` for mutually exclusive row, tab, picker, and command
  selection.
- Use `KeyedSubViews` in host entities for stateful or heavy repeated rows.
- Keep `ForEach` focused on lightweight element lists; row entity caching lives
  in host entities through `KeyedSubViews`.
- Use `Resource::reload` / `latest` for async data surfaces that should keep
  stale-ready content visible while refreshing. Prefer local render branches
  until repeated real app surfaces justify a shared boundary helper; the current
  evidence is still the gallery output surface plus relay examples.
- Use `batch` or single-operation collection helpers such as
  `SignalVecExt::extend` for write bursts that should notify once. Avoid
  assuming a global frame-boundary batch when code does not have a `Window`
  lifecycle hook.

## Next Landing Steps

1. Exercise the new command/picker selector adapters in a second app-shaped
   surface before adding broader picker host abstractions. The Relay primitive
   should remain `Selector<K>` plus host-owned key order.
2. Add a shared async UI boundary only after at least two real app surfaces
   repeat the same `fold_latest` render shape. Current evidence is still one
   gallery output surface plus examples, so `Resource` remains a UI-agnostic
   state primitive.
3. Revisit show/switch style helpers only after there is repeated app code that
   needs persistent branch state.
4. Keep expanding compiled app-shaped surfaces before adding new Relay
   primitives. The workbench migration did not require a new UIKit adapter:
   existing `selected_by` / `active_by` hooks were enough once state lived in
   `Selector<K>` and row retention lived in host-owned `KeyedSubViews`. Its
   selected-item projections also stayed inside ordinary `Memo`, so defer a
   selector-item helper until at least one more surface repeats the same shape
   with enough boilerplate to justify it.

## Verification Gates

- `cargo test -p relay`
- `cargo check -p relay --examples`
- `cargo test -p relay_uikit`
- `cargo check -p relay_uikit --bin relay_gallery`
- `cargo check --workspace`
- `git diff --check`
