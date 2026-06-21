# relay

[English](README.md) | 简体中文

`relay` 是 [GPUI](https://github.com/zed-industries/zed) 的响应式状态运行时层。它提供 signal、派生状态、effect、binding、异步 resource、响应式集合、声明式副作用、跨层 context 和表单聚合——把状态读取记录到当前 GPUI entity，并在状态写入时通过 GPUI 的 `cx.notify` 路径触发刷新。

## 定位

- **面向 GPUI**：API 显式接收 `App` / `Context`，生命周期和刷新都跟随 GPUI。
- **状态优先**：核心是 `Signal<T>`、`Memo<T>`、`Effect`、`Resource<T, E>` 和 `Binding<T>`。
- **UI 线程优先**：默认使用单线程状态模型，适配 GPUI 渲染和前台任务。
- **可被上层组件适配**：组件层可以把 `Binding` / `Resource` 接到具体控件，运行时本身只负责状态和调度。

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
- **`watch(cx, sources, react)`** — 声明式副作用。`sources` 闭包读取依赖，`react` 闭包执行副作用，分离声明与执行。对标 Vue `watch`。
- **`SignalVecExt`** — `Signal<Vec<T>>` 的增量 API：`push` / `insert` / `remove` / `remove_first` / `retain` / `clear` / `set_all`，每个操作走正常通知路径。
- **`SubView`** — 稳定的 GPUI 子 Entity 包装。把有状态或较重的区域拆到自己的 `Entity` 中，再通过 GPUI 的 `AnyView::cached` 路径渲染。
- **`KeyedSubViews`** — 面向列表形态 view 的 keyed row/entity 保持器。按稳定 key 对齐 item 顺序，复用已有 row entity，丢弃移除的 row，并让未变化的兄弟 row 继续复用 GPUI view cache。
- **`provide_context` / `use_context`** — 响应式 provide/inject。基于 GPUI global + SignalId，跨层共享响应式状态（主题、locale、active entity 等），值变化自动通知所有 `use_context` 消费者。
- **`Form`** — 表单聚合模型。注册多个 `Binding<T>` 字段，提供 `is_dirty()`（返回 `Memo<bool>`）、`reset(cx)`、`commit(cx)` 等派生能力。适合设置面板、编辑表单等需要脏检查/重置/提交的场景。
- **`WindowSignalExt::use_signal` / `use_binding`** — 组件内 hooks，供 `RenderOnce` 组件使用。通过 `window.use_keyed_state` 按 `ElementId` 持久化跨渲染状态。对标 React `useState` / Solid `createSignal`。
- **`#[derive(Reactive)]`** (relay_macros) — 字段级响应。将普通结构体转换为生成的 `ReactiveFoo` 包装，字段默认包装为 `Signal<T>`，并提供 `from(cx, value)`、`snapshot(cx)`、`set(cx, value)` 和字段访问器。嵌套结构字段可用 `#[reactive(nested)]` 标记，保留嵌套字段级追踪。

## 应用层范式

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

        // 声明式副作用：count 变化时派生事件字符串
        let _ = cx.watch(
            |cx| { let _ = count.get(cx); },
            move |cx| { /* 例如更新一个 label signal */ },
        );

        // 表单聚合：注册字段，派生 is_dirty
        let mut form = Form::new();
        form.field("enabled", enabled.clone(), cx);
        let settings_dirty = form.build_is_dirty(cx);
        std::mem::forget(form);

        // 提供响应式 context 供跨层共享
        let _ = provide_context(cx, "default-theme".to_string());

        Self { enabled, count, todos, settings_dirty }
    }

    fn add_todo(&self, text: String, cx: &mut App) {
        self.todos.push(cx, text); // 响应式集合操作，自动通知
    }
}

// 在子组件中（无需层层传参）：
fn child_render(cx: &App) {
    let theme = use_context::<String>(cx); // 自动订阅，值变化时通知
}
```

## 异步资源

`Resource::load` 会重置为 `Pending` 并开始加载。`Resource::reload` 会把上一份 ready 值保留为 `Reloading(value)`，让 view 可以继续展示最新可用数据，同时表达刷新进度。UI 需要“最后一份可用值”语义时，用 `state.latest()` 或 `resource.latest(cx)`。

```rust
resource.reload(cx, |cx| async move {
    let value = fetch(cx).await?;
    Ok(value)
});

let state = resource.get(cx);
let latest = state.latest();
```

## Entity 粒度 UI

Relay 的 UI 粒度跟随 GPUI 的 `Entity` 缓存边界。把昂贵区域拆成 `SubView<T>` 字段，用 `cached(...)` 渲染；当列表行本身有状态或重绘成本较高时，用 `KeyedSubViews` 保持 row entity 稳定。

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

## 示例

每个示例演示一个特定的 API 或范式。用 `cargo run -p relay --example <名称>` 运行：

| 示例 | 覆盖 |
|---|---|
| `counter` | `Signal`、`Memo`、`tracked` 渲染 |
| `binding` | `Binding` 双向绑定 |
| `untrack` | `untrack`、`set_silent` / `update_silent` |
| `effect` | `Effect`、`effect_in` entity 作用域 effect |
| `derived` | `derived` / `memo` 派生值 |
| `watch` | `watch` 声明式副作用 |
| `signal_vec` | `SignalVecExt` 响应式列表操作 |
| `resource` | `Resource` 异步 pending/reloading/ready/error 和 latest 值 |
| `context` | `provide_context` / `use_context` 跨层共享 |
| `form` | `Form` 聚合、`is_dirty`、`reset`、`commit` |
| `component_hooks` | `WindowSignalExt::use_signal` — 组件内 hooks |
| `reactive_struct` | `#[derive(Reactive)]` — 字段级响应 |
| `subview` | `SubView` cached 子 Entity 拆分 |
| `keyed_subviews` | `KeyedSubViews` 稳定 row entity |

```sh
cargo run -p relay --example counter
```
