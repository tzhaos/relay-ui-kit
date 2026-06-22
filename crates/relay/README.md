# relay

`relay` is a reactive state runtime layer for [GPUI](https://github.com/zed-industries/zed). It provides signals, derived state, effects, bindings, async resources, reactive collections, declarative side effects, cross-layer context, and form aggregation — recording signal reads to the current GPUI entity and triggering refreshes through GPUI's `cx.notify` path on writes.

## Design

- **GPUI-native**: APIs explicitly take `App` / `Context`; lifecycle and refresh follow GPUI.
- **State-first**: core primitives are `Signal<T>`, `Memo<T>`, `Effect`, `Resource<T, E>`, and `Binding<T>`.
- **UI-thread-first**: single-threaded state model by default, suited to GPUI rendering and foreground tasks.
- **Adaptable by upper layers**: component crates can wire `Binding` / `Resource` to concrete controls; the runtime itself only handles state and scheduling.

See [ADAPTATION_PLAN.md](ADAPTATION_PLAN.md) for the current UIKit migration path and Relay's app-facing landing sequence, and [MIGRATION_PLAN.md](MIGRATION_PLAN.md) for the completion audit plus migration checklist.

## Minimal usage

```rust
use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
use relay::{ReactiveAppExt, ReactiveContextExt, Signal, init};

struct Counter {
    count: Signal<i32>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            count: cx.signal(0),
        }
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| div().child(self.count.get(cx).to_string()))
    }
}
```

## GPUI convenience API

`ReactiveAppExt` adds creation methods to `App` / `Context`:

```rust
let count = cx.signal(0);
let enabled = cx.binding(false);
let doubled = cx.memo({
    let count = count.clone();
    move |cx| count.get(cx) * 2
});
```

`ReactiveContextExt` adds entity-scoped usage to GPUI views:

```rust
cx.tracked(|cx| {
    div().child(count.get(cx).to_string())
});
```

UIKit components can receive `Binding<T>` for two-way binding; the underlying layer still goes through GPUI's element and event system.

## Application-layer primitives

Beyond `Signal` / `Binding` / `Memo` / `Effect` / `Resource`, relay provides these application-layer convenience primitives:

- **`untrack(cx, |cx| ...)`** — read signals within a scope without establishing dependencies. Replaces the `get_untracked()` anti-pattern for "read snapshot but don't subscribe". Also exposed as `cx.untrack(...)`.
- **`Signal::update_silent` / `set_silent`** — silent writes that don't notify dependents. For effect write-back, internal coordination, and avoiding ping-pong. `Binding` has the same methods.
- **`derived`** — semantic alias for `memo`, emphasizing "derived value". Register derived computation in `new()` with `cx.derived(|cx| ...)`, read via `derived.get(cx)` in render; recomputes only when dependencies change.
- **`watch(cx, sources, react)`** — declarative side effects. `sources` reads dependencies; `react` runs in `untrack`, so side-effect reads do not become new sources.
- **`watch_changes(cx, sources, react)`** — same source/react split, but skips the initial reaction. Use it when the initial visible state is already seeded and only later source changes should reload or sync.
- **`effect_with_cleanup` / `effect_in_with_cleanup`** — per-run cleanup for source-dependent side effects. Register cleanup work with `CleanupScope::on_cleanup`; relay runs it before the effect re-runs and when the effect is disposed or the owning GPUI entity is released. Cleanup reads are untracked, while cleanup writes still notify normally.
- **`StateScope`** — entity-owned handle storage for scoped effects, source-driven resource watchers, and dirty-check-only forms. Store it as a view field; entity-scoped effects release through GPUI `on_release`, while app-scoped effects should keep an explicit `Effect` handle when manual disposal is needed.
- **`StateScope::load_resource_from_source(cx, resource, source, build_load)`** — entity-scoped source resource load. `source` runs tracked and returns the exact snapshot passed to `build_load`; the first run calls `Resource::load`, later source changes call `Resource::reload`.
- **`StateScope::reload_resource_from_source(cx, resource, source, build_load)`** — source snapshot variant for ready-seeded resources. It skips the initial reaction, then reloads from the tracked source snapshot after changes.
- **`StateScope::load_resource_on_changes(cx, resource, sources, build_load)`** — entity-scoped source-driven resource load. The first run records sources and starts `Resource::load`; later source changes call `Resource::reload` so the latest ready value remains visible while refreshing.
- **`StateScope::reload_resource_on_changes(cx, resource, sources, build_load)`** — entity-scoped source-driven resource reload. `sources` declares dependencies, `build_load` snapshots current app state after a source change, and the resource reload keeps the latest ready value visible while async work runs.
- **`SignalVecExt`** — incremental API for `Signal<Vec<T>>`: `push` / `extend` / `insert` / `remove` / `remove_first` / `remove_selected_by` / `retain` / `clear` / `set_all`, each going through the normal notification path. Use `extend` when appending multiple items should trigger one reactive notification. Use `remove_selected_by` when a selector-backed list should remove the selected item and reconcile stale selection in one batched operation.
- **`Selector<K>`** — keyed selection state. Rows call `selector.is_selected(cx, key)` to track only their own key; changing selection notifies the previous and next selected keys instead of every row. Hosts can call `selector.reconcile_keys(cx, keys)` when a list changes to drop stale row signals and clear a selected key that no longer exists, and `select_next` / `select_previous` / `select_first` / `select_last` for ordered list navigation. Use the `_by` variants when the host has item structs and wants to map each item to its stable key without cloning the whole list first. Command/picker-like surfaces can usually stay as host-owned item order plus `Selector<K>` instead of a Relay-level command registry.
- **`SelectedItemExt`** — selected item projection for selector-backed collections. Call `items.selected_by(cx, selector, |item| item.id)` on `Signal<Vec<T>>` or `Memo<Vec<T>>` to derive `Memo<Option<T>>`; use `selected_by_or_first` when the app wants first-item fallback without mutating the selector.
- **`SubView`** — stable GPUI child entity wrapper. Use it to split stateful or heavy regions into their own `Entity` and render them with GPUI's `AnyView::cached` path.
- **`KeyedSubViews`** — keyed row/entity retention for list-shaped views. Reconciles item order by stable key, reuses existing row entities, drops removed rows, and lets clean sibling rows reuse GPUI view cache. Use `sync_with_selector` when a retained row list is also driven by a `Selector<K>`; it reconciles stale selection before syncing row entities.
- **`provide_context` / `use_context`** — reactive provide/inject. Based on GPUI global + SignalId; shares reactive state across layers (theme, locale, active entity). Value changes notify all `use_context` consumers automatically.
- **`Form`** — form aggregation model. Register multiple `Binding<T>` fields; provides `is_dirty()` (returns `Memo<bool>`), `reset(cx)`, and `commit(cx)`. Suited for settings panels, edit forms, and other dirty-check/reset/submit scenarios.
- **`StateScope::form()`** — entity-scoped form builder. Use it for dirty-check-only forms so the owning view keeps the form lifetime without `std::mem::forget`. Store `Form` directly when the view needs `reset(cx)` or `commit(cx)`.
- **`WindowSignalExt::use_signal` / `use_binding`** — component-internal hooks for `RenderOnce` components. Calls `window.use_keyed_state` to persist state across renders keyed by `ElementId`. The React `useState` / Solid `createSignal` equivalent for GPUI.
- **`#[derive(Reactive)]`** (relay_macros) — field-level reactivity. Transforms a plain struct into a generated `ReactiveFoo` wrapper with `Signal<T>` fields, `from(cx, value)`, `snapshot(cx)`, `set(cx, value)`, and generated field accessors. Mark nested struct fields with `#[reactive(nested)]` to keep their own field-level tracking.

## Application-layer patterns

```rust
use relay::{
    Binding, Memo, ReactiveAppExt, ReactiveContextExt, Signal, SignalVecExt, StateScope,
    provide_context, use_context,
};

struct SettingsView {
    enabled: Binding<bool>,
    count: Signal<i32>,
    todos: Signal<Vec<String>>,
    settings_dirty: Memo<bool>,
    scope: StateScope,
}

impl SettingsView {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let mut scope = StateScope::new();
        let enabled = cx.binding(false);
        let count = cx.signal(0);
        let todos: Signal<Vec<String>> = cx.signal(Vec::new());

        // Declarative side effect: derive an event string when count changes.
        let _ = cx.watch(
            |cx| { let _ = count.get(cx); },
            move |cx| { /* e.g. update a label signal */ },
        );

        // Source-dependent side effect: clean the previous handle before
        // subscribing to the next source.
        let channel = cx.signal("inbox");
        let channel_for_effect = channel.clone();
        let _ = cx.effect_in_with_cleanup(move |cx, cleanup| {
            let name = channel_for_effect.get(cx);
            cleanup.on_cleanup(move |_cx| {
                // close listener/subscription for `name`
            });
        });

        // Form aggregation: register fields, derive is_dirty.
        let settings_dirty = scope
            .form()
            .field("enabled", enabled.clone(), cx)
            .build_is_dirty(cx);

        // Provide a reactive context for cross-layer sharing.
        let _ = provide_context(cx, "default-theme".to_string());

        Self { enabled, count, todos, settings_dirty, scope }
    }

    fn add_todo(&self, text: String, cx: &mut App) {
        self.todos.push(cx, text); // reactive collection op, auto-notifies
    }

    fn add_default_todos(&self, cx: &mut App) {
        self.todos.extend(cx, ["Plan".to_string(), "Build".to_string()]);
    }
}

// In a child component (no prop drilling):
fn child_render(cx: &App) {
    let theme = use_context::<String>(cx); // auto-subscribes, notified on change
}
```

## Async resources

`Resource::load` starts a reset load and enters `Pending`. `Resource::reload` keeps a previous ready value available as `Reloading(value)`, so views can keep rendering the latest data while showing refresh progress. Use `state.latest()` or `resource.latest(cx)` when the UI wants "last usable value" semantics. Use `is_loading(cx)`, `has_latest(cx)`, `read_error(cx, ...)`, and `error_value(cx)` for signal-backed status reads without matching the whole state. Use `fold_latest` when a view wants to handle pending, latest-value, and error branches without repeating the `Ready` / `Reloading` match.

Relay intentionally stops at resource state and folding semantics. When two concrete surfaces share the same render-ready shape, put that adapter in the component crate; when a surface needs its own metadata or rows, fold the resource locally.

For source-driven resources, keep the resource UI-agnostic and wire the source
through an entity-scoped scope. Use the `_from_source` helpers when the async
load inputs are exactly the tracked source snapshot. Use `load_resource_from_source`
when the initial value should be loaded asynchronously; use
`reload_resource_from_source` when a ready initial value has already been
installed:

```rust
scope.load_resource_from_source(
    cx,
    output.clone(),
    move |cx| selected_task.get(cx),
    move |task| {
        move |cx| async move {
            let value = fetch_output(cx, task).await?;
            Ok(value)
        }
    },
);
```

```rust
scope.reload_resource_from_source(
    cx,
    output.clone(),
    move |cx| selected_task.get(cx),
    move |task| {
        move |cx| async move {
            let value = fetch_output(cx, task).await?;
            Ok(value)
        }
    },
);
```

Use `load_resource_on_changes` / `reload_resource_on_changes` when the tracked
source declaration and load construction intentionally differ.

```rust
resource.reload(cx, |cx| async move {
    let value = fetch(cx).await?;
    Ok(value)
});

let state = resource.get(cx);
let latest = state.latest();
let loading = resource.is_loading(cx);

let label = resource.fold_latest(
    cx,
    || "Loading".to_string(),
    |value, reloading| {
        if reloading { format!("{value} (refreshing)") } else { value.clone() }
    },
    |error| format!("Failed: {error}"),
);

let error_label = resource.read_error(cx, |error| error.map(|error| error.to_string()));
```

## Entity-grained UI

Relay's UI-level granularity follows GPUI's `Entity` cache boundary. Split expensive regions into `SubView<T>` fields, render them with `cached(...)`, and keep list rows stable with `KeyedSubViews` when the row has state or is expensive to redraw.

Use lightweight element mapping for cheap stateless rows. Move a row to `SubView` / `KeyedSubViews` when it owns state, focus or scroll-like element state, async resources, scoped effects, or enough rendering work that clean siblings should stay cached. This keeps Relay aligned with GPUI's real lifecycle: element helpers rebuild elements during parent render, while `KeyedSubViews` retains child entities by stable key.

Persistent branches, such as tabs, panes, and view modes, follow the same rule:
keep each stateful branch as a `SubView` field in the host and render the active
branch. GPUI `hidden()` sets `display: none`; those children are skipped during
layout, prepaint, and paint, so use it for presentation rather than branch
lifetime. See the `branch_subviews` example for the retained-branch pattern.

```rust
struct TaskList {
    rows: KeyedSubViews<u64, TaskRow>,
    tasks: Signal<Vec<Task>>,
}

impl ReactiveView for TaskList {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tasks = self.tasks.get(cx);
        self.rows.sync(
            cx,
            tasks,
            |task| task.id,
            |task, _cx| TaskRow::new(task),
            |task, row, _cx| row.update_task(task),
        );

        div()
            .children(self.rows.cached(gpui::StyleRefinement::default().w_full()))
            .into_any_element()
    }
}
```

For selection-heavy lists, pair row entities with `Selector<K>` so each row
tracks only its own selected state:

```rust
let selected = cx.selector(Some(1_u64));
let active = selected.is_selected(cx, row_id);
selected.select(cx, next_id);
selected.select_next(cx, tasks.iter().map(|task| task.id));
selected.select_previous(cx, tasks.iter().map(|task| task.id));
selected.select_first(cx, tasks.iter().map(|task| task.id));
selected.select_last(cx, tasks.iter().map(|task| task.id));
selected.reconcile_keys(cx, tasks.iter().map(|task| task.id));

// For item collections, use the `_by` variants to keep key extraction local.
selected.select_next_by(cx, &tasks, |task| task.id);
selected.reconcile_keys_by(cx, &tasks, |task| task.id);
```

When a retained row list is selector-backed, sync the selector and row entities
through the same key function:

```rust
rows.sync_with_selector(
    cx,
    &selected,
    tasks,
    |task| task.id,
    |task, _cx| TaskRow::new(task, selected.clone()),
    |task, row, _cx| row.update_task(task),
);
```

When a view also needs the selected item, derive it from the same collection
and selector:

```rust
let selected_task = tasks.selected_by_or_first(cx, selected.clone(), |task| task.id);
let selected_command = visible_commands.selected_by(cx, command_selector, |command| command.id);
```

When a host command removes the selected item from a selector-backed list, use
the collection helper so the list and selected key cannot drift between the
write and the next render:

```rust
tasks.remove_selected_by(cx, &selected, |task| task.id);
```

## Examples

Each example demonstrates a specific API or pattern. Run with `cargo run -p relay --example <name>`:

| Example | Covers |
|---|---|
| `counter` | `Signal`, `Memo`, `tracked` render |
| `binding` | `Binding` two-way binding |
| `untrack` | `untrack`, `set_silent` / `update_silent` |
| `effect` | `Effect`, `effect_in` entity-scoped effects |
| `effect_cleanup` | `effect_in_with_cleanup` per-run side-effect cleanup |
| `derived` | `derived` / `memo` derived values |
| `watch` | `watch` / `watch_changes` declarative side effects |
| `signal_vec` | `SignalVecExt` reactive list operations |
| `resource` | `Resource` async pending/reloading/ready/error and latest value |
| `source_resource` | `StateScope` source-driven resource load/reload helpers |
| `context` | `provide_context` / `use_context` cross-layer sharing |
| `form` | `Form` aggregation, `is_dirty`, `reset`, `commit` |
| `component_hooks` | `WindowSignalExt::use_signal` — component-internal state |
| `reactive_struct` | `#[derive(Reactive)]` — field-level reactivity |
| `subview` | `SubView` cached child entity splitting |
| `branch_subviews` | Persistent branch/panel state with host-owned `SubView`s |
| `keyed_subviews` | `KeyedSubViews` retained row entities with `Selector` navigation |
| `command_picker` | Command/picker-style host state with `Binding`, `Memo`, and `Selector` |
| `session_surface` | GPUI session surface with retained rows and host-level keyboard navigation |

```sh
cargo run -p relay --example counter
```
