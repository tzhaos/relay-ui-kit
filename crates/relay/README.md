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

运行示例：

```sh
cargo run -p relay --example counter
```
