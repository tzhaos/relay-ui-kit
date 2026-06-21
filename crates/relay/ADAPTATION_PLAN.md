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
- `Resource::load`, `Resource::reload`, `latest`, and `fold_latest` cover
  async pending, reloading, ready, and error states without baking in a UI
  boundary.
- `SubView` and `KeyedSubViews` expose GPUI entity-grained UI retention.
- `Selector<K>` gives selection-heavy lists per-key tracking, can reconcile
  selection against the current item keys, and owns ordered next/previous
  navigation for list and command surfaces.
- `#[derive(Reactive)]` supports nested reactive state wrappers for field-level
  app state.
- `relay_uikit` has begun consuming `Selector<K>` for task/session/tab
  selection, including keyed gallery hosts where selection is host-owned rather
  than embedded in row data.
- The gallery Patterns output surface consumes `Resource::reload` / `latest`
  semantics through `fold_latest`, keeping the previous output visible while an
  async refresh is in flight.
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
- The compiled `keyed_subviews` Relay example now pairs `KeyedSubViews` with
  `Selector<u64>` and host-level arrow-key handling. Its tests verify row
  entity reuse while selection moves, previous-selection wraparound, and Enter
  acting on the selected row.

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
  until repeated call sites justify a shared boundary helper.
- Use `batch` or single-operation collection helpers such as
  `SignalVecExt::extend` for write bursts that should notify once. Avoid
  assuming a global frame-boundary batch when code does not have a `Window`
  lifecycle hook.

## Next Landing Steps

1. Promote the verified `keyed_subviews` shape to the first real
   workbench/session surface outside gallery that owns dynamic item
   collections. Keep presentation components value-first; use keyed entity
   retention only when rows have state or enough render cost. The existing
   `workbench_demo.rs` draft is not a completion target until it is wired into
   a compiled binary with its missing modules present.
2. Wire keyboard/command-list navigation at host level with
   `Selector::select_next` / `select_previous` when a real focusable command or
   picker surface needs it. Only add UIKit adapters if repeated call sites prove
   the component API needs them.
3. Add a shared async UI boundary only after at least two app surfaces repeat
   the same `fold_latest` render shape. Until then, `Resource` should remain a
   UI-agnostic state primitive.
4. Revisit show/switch style helpers only after there is repeated app code that
   needs persistent branch state.

## Verification Gates

- `cargo test -p relay`
- `cargo check -p relay --examples`
- `cargo test -p relay_uikit`
- `cargo check -p relay_uikit --bin relay_gallery`
- `cargo check --workspace`
- `git diff --check`
