# Relay Core Audit and Migration Plan

Relay is ready to serve as GPUI's app-state runtime layer when app code follows
the GPUI-shaped boundaries below: state and effects live in relay, rendering and
element lifecycle stay in GPUI, and component crates adapt relay values only at
their public control surfaces.

## Completion Audit

| Requirement | Current evidence | Status |
|---|---|---|
| Reactive entity tracking uses GPUI invalidation | `runtime.rs` tracks signal reads to the current `EntityId`, `reactive_render` wraps `render_state` in `cx.tracked`, and tests cover dependency replacement, entity release cleanup, `untrack`, and batching. | Complete |
| State primitives cover app data flow | `Signal`, `ReadSignal`, `WriteSignal`, `Binding`, `Memo`, `derived`, `batch`, and `SignalVecExt` are exported and covered by unit tests plus `counter`, `binding`, `derived`, `signal_vec`, and `untrack` examples. `SignalVecExt::remove_selected_by` covers the repeated selector-backed removal shape without adding a collection store. | Complete |
| Side effects have GPUI-scoped lifetime and cleanup | `effect_in`, `effect_in_with_cleanup`, `StateScope::effect_in_with_cleanup`, and cleanup tests cover rerun cleanup, dispose cleanup, entity release cleanup, StateScope-held entity cleanup, and untracked cleanup reads. The `effect_cleanup` example covers source-dependent subscription switching. | Complete |
| Declarative source/react split is available | `watch` and `watch_changes` track only declared sources and run reactions untracked. `hooks.rs`, `view.rs`, and the `watch` example cover source-only dependencies and skipped initial reactions. | Complete |
| Async resource state fits app surfaces without UI ownership | `Resource::load`, `reload`, `latest`, status helpers, `fold_latest`, and stale-ready reload semantics are tested in `resource.rs`. `StateScope::load_resource_on_changes` and `reload_resource_on_changes` are covered in `view.rs`, `source_resource`, and workbench transcript/review tests. | Complete |
| Entity-grained UI retention follows GPUI cache boundaries | `SubView`, `KeyedSubViews`, and `KeyedSubViews::sync_with_selector` keep stateful regions and rows as GPUI entities. Tests cover cached sibling reuse, retained branches, row reuse, row-local state survival, and stale selection reconciliation. | Complete |
| Selection-heavy lists avoid whole-list invalidation | `Selector<K>` provides per-key selected signals, ordered navigation, key reconciliation, and `_by` helpers. `SelectedItemExt` covers selected-item projection from `Signal<Vec<T>>` and `Memo<Vec<T>>`; examples and gallery/workbench call sites use it. | Complete |
| Field-level state wrappers are available | `#[derive(Reactive)]` supports nested reactive state wrappers; `tests/reactive_derive.rs` covers nested field tracking and snapshots. | Complete |
| Form and context patterns are scoped | `Form`, `StateScope::form`, `provide_context`, and `use_context` are exported and covered by unit tests plus README usage. | Complete |
| UIKit adaptation path is narrow and value-first | `relay_uikit` consumes `Binding`, `Selector`, `Resource::fold_latest`, `SelectedItemExt`, `KeyedSubViews`, and `sync_with_selector` in compiled gallery/workbench surfaces without moving command registries, resource UI boundaries, or row presentation into relay. | Complete |

## Migration Plan

### 1. Inventory Host State

For each GPUI view or app surface, classify state by ownership:

- Local scalar or app state: use `Signal<T>` or `Binding<T>`.
- Derived state: use `Memo<T>` / `cx.derived`.
- Two-way control value: expose `Binding<T>` to `relay_uikit`.
- Dynamic collection: use `Signal<Vec<T>>` plus `SignalVecExt` for mutations.
- Selection over stable keys: use `Selector<K>`.
- Async state: use `Resource<T, E>`.
- Effects, resources, and form lifetimes owned by an entity: keep a
  `StateScope` field.

Keep presentation structs and row data in the host or component crate. Relay
should not own command registries, output-log render adapters, or UIKit-specific
row layout.

Current inventory checkpoint:

| Surface | Host-owned relay state | UIKit boundary |
|---|---|---|
| `relay_gallery::GalleryApp` | `Signal<Page>` and `Signal<bool>` for route/theme shell state; child scene entities stay cached behind GPUI entities. | Navigation rows, title bar, and shell layout receive values/callbacks only. |
| `GalleryScenesApp` | `Binding` values for controls, `Selector` values for mutually exclusive rows/tabs/commands, `Resource<Vec<OutputLine>, String>` for output refresh, `Memo` for derived tree/dirty state, and `Signal<Vec<FeedbackToast>>` for notifications. | Scenes pass `Binding`, selector-backed adapters, resource snapshots, and value structs into components. |
| `WorkbenchApp` | `Signal<Vec<_>>` collections, selected task/session `Selector<u64>`, selected-item `Memo`s, `Resource` values for transcript/review, and a `StateScope` for source-driven reloads. | Rail/context rows use `KeyedSubViews`; output surfaces consume folded resource values. |
| Retained gallery/workbench rows | Row-local state is stored inside row `Entity`s and synchronized by `KeyedSubViews::sync_with_selector`. | Row presentation stays in `relay_uikit` patterns or gallery host modules. |
| Small relay examples | Simple state demos use direct `cx.tracked` where the example is about a primitive; app-shaped examples use `ReactiveView`. | No UIKit dependency. |

### 2. Move Renders to Reactive Tracking

For each stateful GPUI entity:

```rust
impl ReactiveView for MyView {
    fn render_state(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        // Read signals here.
    }
}

impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}
```

Use `cx.tracked` directly only for small one-off views. Use `reactive_render`
for ordinary app entities so signal dependencies are refreshed each render.

Current migration checkpoint:

- `GalleryApp`, `GalleryScenesApp`, `WorkbenchApp`, keyed row hosts, retained
  branch examples, source-resource examples, and command/picker examples render
  through `ReactiveView` plus `reactive_render`.
- Small primitive examples keep explicit `cx.tracked` so the lower-level API
  remains documented by runnable code.
- Plain host fields that affect a cached child scene should call `cx.notify()`
  when changed; signal-backed fields already notify through Relay.

### 3. Migrate Selection and Lists

For lightweight stateless rows, map elements directly from `Signal<Vec<T>>` or
use the component crate's `ForEach`.

For rows with local state, focus-like element state, resources, effects, or
meaningful render cost, move rows behind `KeyedSubViews`:

```rust
let items = self.items.get(cx);
self.rows.sync_with_selector(
    cx,
    &self.selection,
    items,
    |item| item.id,
    |item, _cx| RowView::new(item, self.selection.clone()),
    |item, row, _cx| row.update_item(item),
);
```

Use `SelectedItemExt` when a pane or status bar needs the selected item:

```rust
let selected_item = items.selected_by_or_first(cx, selection.clone(), |item| item.id);
```

When there are no retained row entities, keep using `Selector::reconcile_keys`
or `reconcile_keys_by` directly after collection changes.

When a host removes the currently selected item, use `SignalVecExt` to keep the
collection write and selector reconciliation atomic:

```rust
items.remove_selected_by(cx, &selection, |item| item.id);
```

Current migration checkpoint:

- Workbench task/session lists and the gallery stress session list use
  `remove_selected_by` for selected-row deletion.
- This replaces repeated host-side `get_untracked + Vec::remove +
  reconcile_keys_by` code while preserving host ownership of item order and row
  presentation.

### 4. Migrate Async Data

Use `Resource::load` for a reset load and `Resource::reload` when the last ready
value should stay visible. In entity-owned source-driven flows:

- Use `StateScope::load_resource_on_changes` when the resource starts pending.
- Use `StateScope::reload_resource_on_changes` when the view already has a ready
  seed value.
- Use `fold_latest` in app code or a narrow adapter in the component crate when
  repeated UI surfaces share the exact same render-ready shape.

Do not put UI fallback components or Suspense-like boundaries in relay unless
multiple real GPUI surfaces converge on the same typed boundary.

Current migration checkpoint:

- Workbench transcript and review resources use
  `StateScope::reload_resource_on_changes` because both start from ready seed
  values and should keep stale-ready content visible while source selections
  change.
- The Relay `source_resource` example uses `StateScope::load_resource_on_changes`
  for the pending-first version of the same source/resource pattern.
- Output-log resources use the narrow `relay_uikit::output_resource_snapshot`
  adapter because the shared render-ready shape is specific to
  `Resource<Vec<OutputLine>, E>`.
- Review data still folds locally with `Resource::fold_latest`; its render
  shape has not repeated enough to justify a Relay or UIKit boundary.
- No `create_resource`/Suspense-like API is currently justified: GPUI entity
  lifetime, `StateScope`, and resource folding cover the real surfaces without
  introducing an owner tree or UI fallback primitive in Relay.

### 5. Migrate Effects and External Handles

Use `watch` / `watch_changes` for source-driven side effects that do not own
temporary external handles. Use `effect_in_with_cleanup` when a side effect owns
subscriptions, timers, listeners, or other handles that must be released before
the next source subscription or on entity release.

Register those handles with `CleanupScope::on_cleanup`; cleanup reads are
untracked and cleanup writes still notify through the normal signal path.

Current migration checkpoint:

- The `effect_cleanup` example verifies source changes clean up the old
  subscription before installing the next one.
- `effect.rs` tests verify cleanup before re-run, cleanup on manual dispose,
  cleanup on entity release, and untracked cleanup reads.
- `view.rs` tests verify `StateScope::effect_in_with_cleanup` still cleans up
  on entity release when the view stores only a `StateScope` field. The
  lifetime path is GPUI `cx.on_release`, not `StateScope::drop`.
- `StateScope::effect` is not the preferred view-lifetime path for new code:
  app-scoped effects should keep an explicit `Effect` handle for disposal, and
  view-owned effects should use `effect_in` / `effect_in_with_cleanup`.
- No `watch_with_cleanup` helper is currently justified. Existing real surfaces
  either need source/react splitting without handles (`watch` /
  `watch_changes`) or handle cleanup tied to GPUI entity lifetime
  (`effect_in_with_cleanup`).

### 6. Migrate Branches and Panels

For persistent tabs, panes, and view modes that own state, store each branch as
a `SubView<T>` field and render only the active branch through `cached(...)`.
Use `hidden()` for presentation only, not as a branch lifetime primitive.

Current migration checkpoint:

- The `branch_subviews` example keeps each branch as a host-owned `SubView`,
  renders only the active branch through GPUI's cached view path, and tests
  inactive branch render suppression plus state survival across switching.
- GPUI's `AnyView::cached` reuses previous prepaint/paint work when the view
  entity is clean and the cache key still matches; `SubView` and
  `KeyedSubViews` align Relay's UI granularity with that entity cache boundary.
- GPUI `hidden()` sets `Display::None` and skipped children do not prepaint or
  paint, so it remains a presentation tool instead of a persistent branch
  lifecycle primitive.
- No Show/Switch helper is currently justified. A helper should wait until
  repeated app surfaces show a common typed branch shape beyond ordinary
  host-owned `SubView` fields.

### 7. Verification Gates

Run these before considering a migration checkpoint complete:

```sh
cargo test -p relay
cargo test -p relay --examples
cargo check -p relay --examples
cargo test -p relay_uikit
cargo check -p relay_uikit --bin relay_gallery
cargo check --workspace
git diff --check
```

Existing `relay_uikit` unused warnings are tracked as gallery/demo hygiene and
do not block relay migration checkpoints unless they hide a changed code path.

## Future Additions Policy

Add a new relay primitive only when at least two compiled app-shaped surfaces
repeat the same GPUI-shaped state pattern and the helper can stay runtime-level.
Prefer documenting a boundary or adding a narrow extension method over adding
SolidJS-like names for their own sake.

Current deferred ideas and their bar:

- Source resource constructor: defer while `StateScope::*resource_on_changes`
  covers entity-scoped source/resource lifetime.
- Show/Switch helper: defer while host-owned `SubView` fields cover persistent
  branch state.
- `watch_with_cleanup`: defer while `effect_in_with_cleanup` covers handle
  lifetime and `watch` remains a simpler source/react split.
- Command registry or selector-backed collection store: defer while
  `Selector`, `SelectedItemExt`, `sync_with_selector`, and
  `SignalVecExt::remove_selected_by` keep command and row data host-owned.
- Frame-boundary automatic batching: defer without a GPUI `Window` lifecycle
  hook that clearly improves real app write bursts beyond explicit `batch` and
  collection helpers.
