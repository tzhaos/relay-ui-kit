# relay

`relay` 是 GPUI 的响应式状态运行时层。它提供 signal、derived state、effect、binding 和 async resource，把状态读取记录到当前 GPUI entity，并在状态写入时通过 GPUI 的 `cx.notify` 路径触发刷新。

## 定位

- 面向 GPUI：API 显式接收 `App` / `Context`，生命周期和刷新都跟随 GPUI。
- 状态优先：核心是 `Signal<T>`、`Memo<T>`、`Effect`、`Resource<T, E>` 和 `Binding<T>`。
- UI 线程优先：默认使用单线程状态模型，适配 GPUI 渲染和前台任务。
- 可被上层组件适配：后续组件层可以把 `Binding` / `Resource` 接到具体控件，但运行时本身只负责状态和调度。

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

除 `Signal` / `Binding` / `Memo` / `Effect` / `Resource` 外，relay 还提供以下
应用层便利原语：

- **`untrack(cx, |cx| ...)`** — 在作用域内读取信号但不建立依赖。适合"读快照但
  不订阅"的场景，替代 `get_untracked()` 反模式。也通过 `cx.untrack(...)` 暴露。
- **`Signal::update_silent` / `set_silent`** — 静默写入，不通知依赖。用于 effect
  回写自身读取的信号、内部协调等避免 ping-pong 的场景。`Binding` 也有同名方法。
- **`derived`** — `memo` 的语义别名，强调"派生值"。在 view 的 `new()` 里用
  `cx.derived(|cx| ...)` 注册派生计算，渲染时 `derived.get(cx)` 读取，依赖变化才重算。
- **`watch(cx, sources, react)`** — 声明式副作用。`sources` 闭包读取依赖，`react`
  闭包执行副作用，分离声明与执行。对标 Vue `watch`。
- **`SignalVecExt`** — `Signal<Vec<T>>` 的增量 API：`push` / `insert` / `remove` /
  `retain` / `clear` / `set_all`，每个操作走正常通知路径。
- **`ForEach`** (relay_uikit) — 响应式列表组件，接收 `Signal<Vec<T>>` + key fn +
  render fn，自动订阅信号刷新。

## 应用层范式

```rust
use relay::{Binding, ReactiveAppExt, ReactiveContextExt, Signal, SignalVecExt};

struct SettingsView {
    enabled: Binding<bool>,
    count: Signal<i32>,
    todos: Signal<Vec<String>>,
}

impl SettingsView {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let enabled = cx.binding(false);
        let count = cx.signal(0);
        let todos: Signal<Vec<String>> = cx.signal(Vec::new());

        // 声明式联动：count 变化时派生 label
        let _ = cx.watch(
            |cx| { let _ = count.get(cx); },
            move |cx| { /* e.g. update a derived label */ },
        );

        Self { enabled, count, todos }
    }

    fn add_todo(&self, text: String, cx: &mut App) {
        self.todos.push(cx, text); // 响应式集合操作，自动通知
    }
}
```

运行示例：

```sh
cargo run -p relay --example counter
```
