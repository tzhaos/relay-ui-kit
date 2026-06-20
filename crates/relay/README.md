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
use relay::{Signal, init, track};

struct Counter {
    count: Signal<i32>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            count: Signal::new(cx, 0),
        }
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        track(cx, |cx| div().child(self.count.get(cx).to_string()))
    }
}
```

运行示例：

```sh
cargo run -p relay --example counter
```
