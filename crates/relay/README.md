# relay

`relay` is a reactive state runtime layer for [GPUI](https://github.com/zed-industries/zed). It provides signals, derived state, effects, bindings, async resources, reactive collections, declarative side effects, cross-layer context, and form aggregation — recording signal reads to the current GPUI entity and triggering refreshes through GPUI's `cx.notify` path on writes.

## Design

- **GPUI-native**: APIs explicitly take `App` / `Context`; lifecycle and refresh follow GPUI.
- **State-first**: core primitives are `Signal<T>`, `Memo<T>`, `Effect`, `Resource<T, E>`, and `Binding<T>`.
- **UI-thread-first**: single-threaded state model by default, suited to GPUI rendering and foreground tasks.
- **Adaptable by upper layers**: component crates can wire `Binding` / `Resource` to concrete controls; the runtime itself only handles state and scheduling.

See [ADAPTATION_PLAN.md](ADAPTATION_PLAN.md) for the current UIKit migration path and Relay's app-facing landing sequence.

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
- **`watch(cx, sources, react)`** — declarative side effects. `sources` closure reads dependencies; `react` closure executes the side effect. Separates declaration from execution. Vue `watch` equivalent.
- **`SignalVecExt`** — incremental API for `Signal<Vec<T>>`: `push` / `extend` / `insert` / `remove` / `remove_first` / `retain` / `clear` / `set_all`, each going through the normal notification path. Use `extend` when appending multiple items should trigger one reactive notification.
- **`Selector<K>`** — keyed selection state. Rows call `selector.is_selected(cx, key)` to track only their own key; changing selection notifies the previous and next selected keys instead of every row. Hosts can call `selector.reconcile_keys(cx, keys)` when a list changes to drop stale row signals and clear a selected key that no longer exists, and `select_next` / `select_previous` / `select_first` / `select_last` for ordered list navigation. Use the `_by` variants when the host has item structs and wants to map each item to its stable key without cloning the whole list first.
- **`SubView`** — stable GPUI child entity wrapper. Use it to split stateful or heavy regions into their own `Entity` and render them with GPUI's `AnyView::cached` path.
- **`KeyedSubViews`** — keyed row/entity retention for list-shaped views. Reconciles item order by stable key, reuses existing row entities, drops removed rows, and lets clean sibling rows reuse GPUI view cache.
- **`provide_context` / `use_context`** — reactive provide/inject. Based on GPUI global + SignalId; shares reactive state across layers (theme, locale, active entity). Value changes notify all `use_context` consumers automatically.
- **`Form`** — form aggregation model. Register multiple `Binding<T>` fields; provides `is_dirty()` (returns `Memo<bool>`), `reset(cx)`, and `commit(cx)`. Suited for settings panels, edit forms, and other dirty-check/reset/submit scenarios.
- **`WindowSignalExt::use_signal` / `use_binding`** — component-internal hooks for `RenderOnce` components. Calls `window.use_keyed_state` to persist state across renders keyed by `ElementId`. The React `useState` / Solid `createSignal` equivalent for GPUI.
- **`#[derive(Reactive)]`** (relay_macros) — field-level reactivity. Transforms a plain struct into a generated `ReactiveFoo` wrapper with `Signal<T>` fields, `from(cx, value)`, `snapshot(cx)`, `set(cx, value)`, and generated field accessors. Mark nested struct fields with `#[reactive(nested)]` to keep their own field-level tracking.

## Application-layer patterns

```rust
use relay::{
    Binding, Form, ReactiveAppExt, ReactiveContextExt, Signal, SignalVecExt,
    provide_context, use_context,
};

struct SettingsView {
    enabled: Binding<bool>,
    count: Signal<i32>,
    todos: Signal<Vec<String>>,
    settings_dirty: Memo<bool>,
}

impl SettingsView {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let enabled = cx.binding(false);
        let count = cx.signal(0);
        let todos: Signal<Vec<String>> = cx.signal(Vec::new());

        // Declarative side effect: derive an event string when count changes.
        let _ = cx.watch(
            |cx| { let _ = count.get(cx); },
            move |cx| { /* e.g. update a label signal */ },
        );

        // Form aggregation: register fields, derive is_dirty.
        let mut form = Form::new();
        form.field("enabled", enabled.clone(), cx);
        let settings_dirty = form.build_is_dirty(cx);
        std::mem::forget(form);

        // Provide a reactive context for cross-layer sharing.
        let _ = provide_context(cx, "default-theme".to_string());

        Self { enabled, count, todos, settings_dirty }
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

`Resource::load` starts a reset load and enters `Pending`. `Resource::reload` keeps a previous ready value available as `Reloading(value)`, so views can keep rendering the latest data while showing refresh progress. Use `state.latest()` or `resource.latest(cx)` when the UI wants "last usable value" semantics. Use `fold_latest` when a view wants to handle pending, latest-value, and error branches without repeating the `Ready` / `Reloading` match.

```rust
resource.reload(cx, |cx| async move {
    let value = fetch(cx).await?;
    Ok(value)
});

let state = resource.get(cx);
let latest = state.latest();

let label = resource.fold_latest(
    cx,
    || "Loading".to_string(),
    |value, reloading| {
        if reloading { format!("{value} (refreshing)") } else { value.clone() }
    },
    |error| format!("Failed: {error}"),
);
```

## Entity-grained UI

Relay's UI-level granularity follows GPUI's `Entity` cache boundary. Split expensive regions into `SubView<T>` fields, render them with `cached(...)`, and keep list rows stable with `KeyedSubViews` when the row has state or is expensive to redraw.

Use lightweight element mapping for cheap stateless rows. Move a row to `SubView` / `KeyedSubViews` when it owns state, focus or scroll-like element state, async resources, scoped effects, or enough rendering work that clean siblings should stay cached. This keeps Relay aligned with GPUI's real lifecycle: element helpers rebuild elements during parent render, while `KeyedSubViews` retains child entities by stable key.

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

## Examples

Each example demonstrates a specific API or pattern. Run with `cargo run -p relay --example <name>`:

| Example | Covers |
|---|---|
| `counter` | `Signal`, `Memo`, `tracked` render |
| `binding` | `Binding` two-way binding |
| `untrack` | `untrack`, `set_silent` / `update_silent` |
| `effect` | `Effect`, `effect_in` entity-scoped effects |
| `derived` | `derived` / `memo` derived values |
| `watch` | `watch` declarative side effects |
| `signal_vec` | `SignalVecExt` reactive list operations |
| `resource` | `Resource` async pending/reloading/ready/error and latest value |
| `context` | `provide_context` / `use_context` cross-layer sharing |
| `form` | `Form` aggregation, `is_dirty`, `reset`, `commit` |
| `component_hooks` | `WindowSignalExt::use_signal` — component-internal state |
| `reactive_struct` | `#[derive(Reactive)]` — field-level reactivity |
| `subview` | `SubView` cached child entity splitting |
| `keyed_subviews` | `KeyedSubViews` retained row entities with `Selector` navigation |
| `session_surface` | GPUI session surface with retained rows and host-level keyboard navigation |

```sh
cargo run -p relay --example counter
```
