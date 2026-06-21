# relay

`relay` is a reactive state runtime layer for [GPUI](https://github.com/zed-industries/zed). It provides signals, derived state, effects, bindings, async resources, reactive collections, declarative side effects, cross-layer context, and form aggregation — recording signal reads to the current GPUI entity and triggering refreshes through GPUI's `cx.notify` path on writes.

## Design

- **GPUI-native**: APIs explicitly take `App` / `Context`; lifecycle and refresh follow GPUI.
- **State-first**: core primitives are `Signal<T>`, `Memo<T>`, `Effect`, `Resource<T, E>`, and `Binding<T>`.
- **UI-thread-first**: single-threaded state model by default, suited to GPUI rendering and foreground tasks.
- **Adaptable by upper layers**: component crates can wire `Binding` / `Resource` to concrete controls; the runtime itself only handles state and scheduling.

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
- **`SignalVecExt`** — incremental API for `Signal<Vec<T>>`: `push` / `insert` / `remove` / `remove_first` / `retain` / `clear` / `set_all`, each going through the normal notification path.
- **`SubView`** — stable GPUI child entity wrapper. Use it to split stateful or heavy regions into their own `Entity` and render them with GPUI's `AnyView::cached` path.
- **`KeyedSubViews`** — keyed row/entity retention for list-shaped views. Reconciles item order by stable key, reuses existing row entities, drops removed rows, and lets clean sibling rows reuse GPUI view cache.
- **`provide_context` / `use_context`** — reactive provide/inject. Based on GPUI global + SignalId; shares reactive state across layers (theme, locale, active entity). Value changes notify all `use_context` consumers automatically.
- **`Form`** — form aggregation model. Register multiple `Binding<T>` fields; provides `is_dirty()` (returns `Memo<bool>`), `reset(cx)`, and `commit(cx)`. Suited for settings panels, edit forms, and other dirty-check/reset/submit scenarios.
- **`WindowSignalExt::use_signal` / `use_binding`** — component-internal hooks for `RenderOnce` components. Calls `window.use_keyed_state` to persist state across renders keyed by `ElementId`. The React `useState` / Solid `createSignal` equivalent for GPUI.
- **`#[derive(Reactive)]`** (relay_macros) — field-level reactivity. Transforms a plain struct into one where each field is wrapped in `Signal<T>`, with generated `get_field`/`set_field`/`update_field`/`signal_field` accessors. Eliminates manual signal creation for multi-field state.

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
}

// In a child component (no prop drilling):
fn child_render(cx: &App) {
    let theme = use_context::<String>(cx); // auto-subscribes, notified on change
}
```

## Async resources

`Resource::load` starts a reset load and enters `Pending`. `Resource::reload` keeps a previous ready value available as `Reloading(value)`, so views can keep rendering the latest data while showing refresh progress. Use `state.latest()` or `resource.latest(cx)` when the UI wants "last usable value" semantics.

```rust
resource.reload(cx, |cx| async move {
    let value = fetch(cx).await?;
    Ok(value)
});

let state = resource.get(cx);
let latest = state.latest();
```

## Entity-grained UI

Relay's UI-level granularity follows GPUI's `Entity` cache boundary. Split expensive regions into `SubView<T>` fields, render them with `cached(...)`, and keep list rows stable with `KeyedSubViews` when the row has state or is expensive to redraw.

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
| `keyed_subviews` | `KeyedSubViews` retained row entities |

```sh
cargo run -p relay --example counter
```
