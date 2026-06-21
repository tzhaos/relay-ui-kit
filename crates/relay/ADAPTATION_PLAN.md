# Relay Adaptation Plan

Relay is GPUI's reactive state/runtime layer. Its app-facing API should make
SolidJS-like state flow convenient while keeping GPUI's entity, element, and
window lifecycles as the source of truth.

## Current Landed Surface

- `Signal`, `Binding`, `Memo` / `derived`, `Effect`, `watch`, `untrack`, and
  reactive context cover app state, derivation, side effects, and cross-layer
  state sharing.
- `Resource::load`, `Resource::reload`, and `latest` cover async pending,
  reloading, ready, and error states.
- `SubView` and `KeyedSubViews` expose GPUI entity-grained UI retention.
- `Selector<K>` gives selection-heavy lists per-key tracking.
- `#[derive(Reactive)]` supports nested reactive state wrappers for field-level
  app state.
- `relay_uikit` has begun consuming `Selector<K>` for task/session/tab
  selection and `KeyedSubViews` in the gallery stress scene.

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
- Use `Resource::reload` / `latest` for future async data surfaces that should
  keep stale-ready content visible while refreshing.

## Next Landing Steps

1. Add one real keyed-row host in an app-like surface beyond the gallery stress
   sample, then verify row entity reuse under update and reorder.
2. Replace boolean row-selection wiring in larger picker/command/list surfaces
   with `Selector<K>` when selection is mutually exclusive.
3. Introduce `Resource` in a real async surface before designing any
   Suspense-like abstraction.
4. Revisit show/switch style helpers only after there is repeated app code that
   needs persistent branch state.

## Verification Gates

- `cargo test -p relay`
- `cargo check -p relay --examples`
- `cargo test -p relay_uikit`
- `cargo check -p relay_uikit --bin relay_gallery`
- `cargo check --workspace`
- `git diff --check`
