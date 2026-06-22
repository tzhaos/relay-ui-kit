# relay

[English](README.md) | 简体中文

`relay` 是 [GPUI](https://github.com/zed-industries/zed) 的响应式状态运行时层。它提供 signal、派生状态、effect、binding、异步 resource、响应式集合、声明式副作用、跨层 context 和表单聚合——把状态读取记录到当前 GPUI entity，并在状态写入时通过 GPUI 的 `cx.notify` 路径触发刷新。

## 定位

- **面向 GPUI**：API 显式接收 `App` / `Context`，生命周期和刷新都跟随 GPUI。
- **状态优先**：核心是 `Signal<T>`、`Memo<T>`、`Effect`、`Resource<T, E>` 和 `Binding<T>`。
- **UI 线程优先**：默认使用单线程状态模型，适配 GPUI 渲染和前台任务。
- **可被上层组件适配**：组件层可以把 `Binding` / `Resource` 接到具体控件，运行时本身只负责状态和调度。

当前 UIKit 迁移路径与 Relay 的应用层落地顺序见 [ADAPTATION_PLAN.md](ADAPTATION_PLAN.md)。

## 最小用法

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

## GPUI 便利 API

`ReactiveAppExt` 给 `App` / `Context` 增加创建方法：

```rust
let count = cx.signal(0);
let enabled = cx.binding(false);
let doubled = cx.memo({
    let count = count.clone();
    move |cx| count.get(cx) * 2
});
```

`ReactiveContextExt` 给 GPUI view 增加 entity-scoped 用法：

```rust
cx.tracked(|cx| {
    div().child(count.get(cx).to_string())
});
```

UIKit 组件可以接收 `Binding<T>` 做双向绑定；底层仍走 GPUI 的元素和事件系统。

## 应用层原语

除 `Signal` / `Binding` / `Memo` / `Effect` / `Resource` 外，relay 还提供以下应用层便利原语：

- **`untrack(cx, |cx| ...)`** — 在作用域内读取信号但不建立依赖。替代 `get_untracked()` 反模式，适合"读快照但不订阅"的场景。也通过 `cx.untrack(...)` 暴露。
- **`Signal::update_silent` / `set_silent`** — 静默写入，不通知依赖。用于 effect 回写自身读取的信号、内部协调等避免 ping-pong 的场景。`Binding` 也有同名方法。
- **`derived`** — `memo` 的语义别名，强调"派生值"。在 view 的 `new()` 里用 `cx.derived(|cx| ...)` 注册派生计算，渲染时 `derived.get(cx)` 读取，依赖变化才重算。
- **`watch(cx, sources, react)`** — 声明式副作用。`sources` 读取依赖，`react` 在 `untrack` 中执行，因此副作用里的读取不会变成新的 source。
- **`watch_changes(cx, sources, react)`** — 同样分离 source/react，但跳过初始 reaction。适合初始可见状态已经 seed 好、只希望后续 source 变化触发 reload 或同步的场景。
- **`StateScope::load_resource_on_changes(cx, resource, sources, build_load)`** — entity 作用域的 source-driven resource load。首次运行记录 source 并触发 `Resource::load`；后续 source 变化触发 `Resource::reload`，刷新时保留最新 ready 值。
- **`StateScope::reload_resource_on_changes(cx, resource, sources, build_load)`** — entity 作用域的 source-driven resource reload。`sources` 声明依赖，`build_load` 在 source 变化后读取当前 app 快照，resource reload 时保留上一份 ready 值继续可见。
- **`SignalVecExt`** — `Signal<Vec<T>>` 的增量 API：`push` / `extend` / `insert` / `remove` / `remove_first` / `retain` / `clear` / `set_all`，每个操作走正常通知路径。批量追加并希望只触发一次响应式通知时，用 `extend`。
- **`Selector<K>`** — keyed 选择状态。行视图用 `selector.is_selected(cx, key)` 只追踪自己的 key；选择项变化时只通知上一个和下一个选中 key，而不是整张列表。列表变化时，host 可以调用 `selector.reconcile_keys(cx, keys)` 丢弃失效行信号，并在当前选中 key 已不存在时清空选择；有序列表导航可以用 `select_next` / `select_previous` / `select_first` / `select_last`。当 host 手里是 item struct 列表时，用 `_by` 变体直接把 item 映射到稳定 key，避免先克隆整张列表。command/picker 一类 surface 通常可以保持为 host 自己拥有 item 顺序，再配 `Selector<K>`，不需要上升成 Relay 级 command registry。
- **`SubView`** — 稳定的 GPUI 子 Entity 包装。把有状态或较重的区域拆到自己的 `Entity` 中，再通过 GPUI 的 `AnyView::cached` 路径渲染。
- **`KeyedSubViews`** — 面向列表形态 view 的 keyed row/entity 保持器。按稳定 key 对齐 item 顺序，复用已有 row entity，丢弃移除的 row，并让未变化的兄弟 row 继续复用 GPUI view cache。
- **`provide_context` / `use_context`** — 响应式 provide/inject。基于 GPUI global + SignalId，跨层共享响应式状态（主题、locale、active entity 等），值变化自动通知所有 `use_context` 消费者。
- **`Form`** — 表单聚合模型。注册多个 `Binding<T>` 字段，提供 `is_dirty()`（返回 `Memo<bool>`）、`reset(cx)`、`commit(cx)` 等派生能力。适合设置面板、编辑表单等需要脏检查/重置/提交的场景。
- **`StateScope::form()`** — entity 作用域的表单 builder。仅需要 dirty-check 的表单用它持有生命周期，避免 `std::mem::forget`；如果 view 需要 `reset(cx)` 或 `commit(cx)`，则直接把 `Form` 存成字段。
- **`WindowSignalExt::use_signal` / `use_binding`** — 组件内 hooks，供 `RenderOnce` 组件使用。通过 `window.use_keyed_state` 按 `ElementId` 持久化跨渲染状态。对标 React `useState` / Solid `createSignal`。
- **`#[derive(Reactive)]`** (relay_macros) — 字段级响应。将普通结构体转换为生成的 `ReactiveFoo` 包装，字段默认包装为 `Signal<T>`，并提供 `from(cx, value)`、`snapshot(cx)`、`set(cx, value)` 和字段访问器。嵌套结构字段可用 `#[reactive(nested)]` 标记，保留嵌套字段级追踪。

## 应用层范式

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

        // 声明式副作用：count 变化时派生事件字符串
        let _ = cx.watch(
            |cx| { let _ = count.get(cx); },
            move |cx| { /* 例如更新一个 label signal */ },
        );

        // 表单聚合：注册字段，派生 is_dirty
        let settings_dirty = scope
            .form()
            .field("enabled", enabled.clone(), cx)
            .build_is_dirty(cx);

        // 提供响应式 context 供跨层共享
        let _ = provide_context(cx, "default-theme".to_string());

        Self { enabled, count, todos, settings_dirty, scope }
    }

    fn add_todo(&self, text: String, cx: &mut App) {
        self.todos.push(cx, text); // 响应式集合操作，自动通知
    }

    fn add_default_todos(&self, cx: &mut App) {
        self.todos.extend(cx, ["Plan".to_string(), "Build".to_string()]);
    }
}

// 在子组件中（无需层层传参）：
fn child_render(cx: &App) {
    let theme = use_context::<String>(cx); // 自动订阅，值变化时通知
}
```

## 异步资源

`Resource::load` 会重置为 `Pending` 并开始加载。`Resource::reload` 会把上一份 ready 值保留为 `Reloading(value)`，让 view 可以继续展示最新可用数据，同时表达刷新进度。UI 需要“最后一份可用值”语义时，用 `state.latest()` 或 `resource.latest(cx)`。状态读取可以用 `is_loading(cx)`、`has_latest(cx)`、`read_error(cx, ...)` 和 `error_value(cx)`，避免为了 loading/error 这类状态匹配整份 state。当 view 只需要处理 pending、latest-value 和 error 三类分支时，用 `fold_latest` 避免重复匹配 `Ready` / `Reloading`。

Relay 到 resource state 和 folding 语义为止。两个具体 surface 共享完全相同的 render-ready 形状时，把适配器放在组件 crate；如果某个 surface 需要自己的 metadata 或 rows，就在本地 fold resource。

source-driven resource 不需要把 UI 边界塞进 `Resource` 本身；在 entity
作用域里把 source 接到 load/reload 即可。初始值需要异步加载时使用
`load_resource_on_changes`；初始 ready 值已经存在时使用
`reload_resource_on_changes`：

```rust
scope.load_resource_on_changes(
    cx,
    output.clone(),
    move |cx| { let _ = selected_task.get(cx); },
    move |cx| {
        let task = selected_task_for_load.get(cx);
        move |cx| async move {
            let value = fetch_output(cx, task).await?;
            Ok(value)
        }
    },
);
```

```rust
scope.reload_resource_on_changes(
    cx,
    output.clone(),
    move |cx| { let _ = selected_task.get(cx); },
    move |cx| {
        let task = selected_task_for_reload.get(cx);
        move |cx| async move {
            let value = fetch_output(cx, task).await?;
            Ok(value)
        }
    },
);
```

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

## Entity 粒度 UI

Relay 的 UI 粒度跟随 GPUI 的 `Entity` 缓存边界。把昂贵区域拆成 `SubView<T>` 字段，用 `cached(...)` 渲染；当列表行本身有状态或重绘成本较高时，用 `KeyedSubViews` 保持 row entity 稳定。

便宜且无状态的 row 可以继续用普通元素映射。row 自己持有状态、focus/scroll 一类 element state、异步 resource、scoped effect，或者渲染成本高到希望干净兄弟 row 复用缓存时，再拆成 `SubView` / `KeyedSubViews`。这样 Relay 的抽象会贴合 GPUI 的真实生命周期：元素 helper 在父 render 中重建元素，而 `KeyedSubViews` 按稳定 key 保持子 entity。

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

对于选择态频繁变化的列表，可以把 row entity 和 `Selector<K>` 配合使用，让每一行只追踪自己的选中状态：

```rust
let selected = cx.selector(Some(1_u64));
let active = selected.is_selected(cx, row_id);
selected.select(cx, next_id);
selected.select_next(cx, tasks.iter().map(|task| task.id));
selected.select_previous(cx, tasks.iter().map(|task| task.id));
selected.select_first(cx, tasks.iter().map(|task| task.id));
selected.select_last(cx, tasks.iter().map(|task| task.id));
selected.reconcile_keys(cx, tasks.iter().map(|task| task.id));

// item 集合可以用 `_by` 变体，把 key 提取留在调用点。
selected.select_next_by(cx, &tasks, |task| task.id);
selected.reconcile_keys_by(cx, &tasks, |task| task.id);
```

## 示例

每个示例演示一个特定的 API 或范式。用 `cargo run -p relay --example <名称>` 运行：

| 示例 | 覆盖 |
|---|---|
| `counter` | `Signal`、`Memo`、`tracked` 渲染 |
| `binding` | `Binding` 双向绑定 |
| `untrack` | `untrack`、`set_silent` / `update_silent` |
| `effect` | `Effect`、`effect_in` entity 作用域 effect |
| `derived` | `derived` / `memo` 派生值 |
| `watch` | `watch` / `watch_changes` 声明式副作用 |
| `signal_vec` | `SignalVecExt` 响应式列表操作 |
| `resource` | `Resource` 异步 pending/reloading/ready/error 和 latest 值 |
| `source_resource` | `StateScope` source-driven resource load/reload helpers |
| `context` | `provide_context` / `use_context` 跨层共享 |
| `form` | `Form` 聚合、`is_dirty`、`reset`、`commit` |
| `component_hooks` | `WindowSignalExt::use_signal` — 组件内 hooks |
| `reactive_struct` | `#[derive(Reactive)]` — 字段级响应 |
| `subview` | `SubView` cached 子 Entity 拆分 |
| `keyed_subviews` | `KeyedSubViews` 稳定 row entity 与 `Selector` 导航 |
| `command_picker` | 用 `Binding`、`Memo`、`Selector` 组合 command/picker 风格 host state |
| `session_surface` | GPUI session surface，覆盖稳定 row entity 与宿主级键盘导航 |

```sh
cargo run -p relay --example counter
```
