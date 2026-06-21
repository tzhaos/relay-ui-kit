# Relay Adaptation Plan

Relay is GPUI's reactive state/runtime layer. Its app-facing API should make
SolidJS-like state flow convenient while keeping GPUI's entity, element, and
window lifecycles as the source of truth.

## Current Landed Surface

- `Signal`, `Binding`, `Memo` / `derived`, `Effect`, `watch`, `untrack`, and
  reactive context cover app state, derivation, side effects, and cross-layer
  state sharing.
- `Resource::load`, `Resource::reload`, `latest`, and `fold_latest` cover
  async pending, reloading, ready, and error states without baking in a UI
  boundary.
- `SubView` and `KeyedSubViews` expose GPUI entity-grained UI retention.
- `Selector<K>` gives selection-heavy lists per-key tracking and can reconcile
  selection against the current item keys.
- `#[derive(Reactive)]` supports nested reactive state wrappers for field-level
  app state.
- `relay_uikit` has begun consuming `Selector<K>` for task/session/tab
  selection and `KeyedSubViews` in the gallery stress scene.
- The gallery Patterns output surface consumes `Resource::reload` / `latest`
  semantics through `fold_latest`, keeping the previous output visible while an
  async refresh is in flight.
- The gallery Patterns item picker now has an app-like keyed host that combines
  `KeyedSubViews` row retention with `Selector<u64>` mutual selection, with
  tests covering row entity reuse across reorder and selected-row updates. The
  picker uses selector reconciliation so removed keys cannot leave stale
  selection behind.

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

## Next Landing Steps

1. Apply `Selector::reconcile_keys` at the next real command/list host that
   owns dynamic item collections. Keep presentation components value-first; use
   keyed entity retention only when rows have state or enough render cost.
2. Add a shared async UI boundary only after at least two app surfaces repeat
   the same `fold_latest` render shape. Until then, `Resource` should remain a
   UI-agnostic state primitive.
3. Revisit show/switch style helpers only after there is repeated app code that
   needs persistent branch state.

## Verification Gates

- `cargo test -p relay`
- `cargo check -p relay --examples`
- `cargo test -p relay_uikit`
- `cargo check -p relay_uikit --bin relay_gallery`
- `cargo check --workspace`
- `git diff --check`
