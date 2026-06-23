# GPUI / Zed Evidence Notes For Relay Design

## Scope

This note records concrete evidence gathered from:

- `F:\Workspace\zed\crates\gpui`
- `F:\Workspace\zed\crates\ui`
- `F:\Workspace\zed\crates\workspace`
- `F:\Workspace\zed\crates\gpui_tokio`

The purpose is to test whether Relay's design should:

- imitate a web-framework runtime closely; or
- align itself with GPUI's actual ownership, rendering, interaction, and async model.

## Evidence 1: Component-level reuse is mostly `RenderOnce`, not entity-backed views

Counts from repository search:

- `ui`: `RenderOnce=70`, `Render=3`, `cx.new=15`
- `workspace`: `RenderOnce=2`, `Render=28`, `cx.new=196`
- `gpui`: `RenderOnce=4`, `Render=77`, `cx.new=127`

Interpretation:

- `ui` is heavily component-oriented and mostly uses `RenderOnce`.
- `workspace` is heavily entity/view-oriented and mostly uses `Render`.
- GPUI itself exposes both modes and uses each where appropriate.

Representative sources:

- GPUI defines `RenderOnce` as the reusable component trait in [element.rs](F:/Workspace/zed/crates/gpui/src/element.rs:179).
- GPUI's own docs recommend components via `RenderOnce` and `#[derive(IntoElement)]` in [element.rs](F:/Workspace/zed/crates/gpui/src/element.rs:30).
- A typical UI component follows this pattern in [button.rs](F:/Workspace/zed/crates/ui/src/components/button/button.rs:78) and [button.rs](F:/Workspace/zed/crates/ui/src/components/button/button.rs:402).

Implication for Relay:

- Relay should not force reusable UI abstraction into entity-backed state by default.
- The right split is:
  - `RenderOnce` for reusable components;
  - `Render` + `Entity<T>` for durable stateful surfaces.

## Evidence 2: GPUI already has a first-class local interaction state mechanism

Key APIs:

- `Window::use_keyed_state` in [window.rs](F:/Workspace/zed/crates/gpui/src/window.rs:3422)
- `Window::use_state` in [window.rs](F:/Workspace/zed/crates/gpui/src/window.rs:3451)
- `Window::with_element_state` in [window.rs](F:/Workspace/zed/crates/gpui/src/window.rs:3467)

Observed usage counts:

- repository-wide `window.use_state(...)`: 12
- repository-wide `with_element_state(...)`: 16

Representative usage:

- temporary resize-highlight state in [redistributable_columns.rs](F:/Workspace/zed/crates/ui/src/components/redistributable_columns.rs:519)
- retained popover element state in [popover_menu.rs](F:/Workspace/zed/crates/ui/src/components/popover_menu.rs:342), [popover_menu.rs](F:/Workspace/zed/crates/ui/src/components/popover_menu.rs:368), and [popover_menu.rs](F:/Workspace/zed/crates/ui/src/components/popover_menu.rs:456)

Interpretation:

- GPUI explicitly distinguishes small, per-element, cross-frame interaction state from entity-owned app state.
- Zed uses this mechanism for hover/resize/menu geometry style problems instead of promoting those concerns into app-level entities or global reactive atoms.

Implication for Relay:

- Relay should document a state taxonomy instead of routing all state through signals.
- Element-local transient state should usually stay in GPUI element state, not Relay signals.

## Evidence 3: Durable workbench surfaces are entity-heavy and lifecycle-rich

Representative source: [pane.rs](F:/Workspace/zed/crates/workspace/src/pane.rs:397)

Relevant lines:

- pane stores durable fields and subscriptions in [pane.rs](F:/Workspace/zed/crates/workspace/src/pane.rs:433)
- pane construction wires focus/global/project subscriptions in [pane.rs](F:/Workspace/zed/crates/workspace/src/pane.rs:544)
- pane itself is a long-lived renderable entity in [pane.rs](F:/Workspace/zed/crates/workspace/src/pane.rs:4291)

Direct evidence from constructor:

- focus hooks
- global settings observation
- project event subscription
- nested entity creation (`toolbar: cx.new(...)`)

Interpretation:

- core desktop/workbench surfaces in Zed are not lightweight component closures.
- they are controllers with focus management, subscriptions, nested entities, and navigation state.

Implication for Relay:

- Relay must respect `Entity<T>` as the real ownership boundary.
- composable logic for desktop app surfaces should be entity-scoped, not purely render-scoped.

## Evidence 4: View caching exists, but is rare and deliberate

Repository search found only 3 `.cached(...)` hits across the inspected crates.

Representative usages:

- dock panel caching in [dock.rs](F:/Workspace/zed/crates/workspace/src/dock.rs:1184) and [dock.rs](F:/Workspace/zed/crates/workspace/src/dock.rs:1185)
- pane caching in [pane_group.rs](F:/Workspace/zed/crates/workspace/src/pane_group.rs:553) and [pane_group.rs](F:/Workspace/zed/crates/workspace/src/pane_group.rs:554)

Interpretation:

- GPUI supports retained view reuse through `AnyView::cached`, but Zed applies it sparingly.
- caching is treated as an optimization boundary for expensive retained surfaces, not as the universal rendering model.

Implication for Relay:

- Relay should expose retained-subview helpers, but not make them the default for every component.
- promote a surface to retained/cached only when lifecycle, focus, async, or rendering cost justify it.

## Evidence 5: High-performance list virtualization is already a GPUI primitive

Representative API:

- `uniform_list(...)` in [uniform_list.rs](F:/Workspace/zed/crates/gpui/src/elements/uniform_list.rs:22)
- `UniformList` in [uniform_list.rs](F:/Workspace/zed/crates/gpui/src/elements/uniform_list.rs:58)
- `UniformListScrollHandle` in [uniform_list.rs](F:/Workspace/zed/crates/gpui/src/elements/uniform_list.rs:80)

Observed usage count:

- repository-wide `uniform_list(...)`: 76

Interpretation:

- Zed relies heavily on GPUI-native list virtualization rather than generalized component reconciliation.
- performance-sensitive UI patterns are handled with dedicated element primitives.

Implication for Relay:

- Relay should not pretend a generic reactive render loop is enough for every UI hotspot.
- for lists, tables, logs, and code-like surfaces, Relay should compose with GPUI performance primitives instead of abstracting over them.

## Evidence 6: Eventing, focus, and window-scoped coordination already live in GPUI

Observed counts across inspected crates:

- `observe`: 112
- `subscribe`: 322
- `spawn_in`: 456

Breakdown:

- `workspace`: `observe=3`, `subscribe=11`, `observe_in=4`, `subscribe_in=10`, `spawn_in=43`, `focus_hooks=9`
- `gpui`: `observe=6`, `subscribe=6`, `spawn_in=1`, `focus_hooks=6`
- `ui`: minimal direct use, as expected for mostly stateless components

Representative GPUI context APIs:

- notify in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:226)
- spawn in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:235)
- listener in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:250)
- defer-in-window in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:301)
- observe-in in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:314)
- subscribe-in in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:343)

Interpretation:

- GPUI already provides a rich event/subscription/focus/window coordination model.
- Zed uses it directly in serious surfaces.

Implication for Relay:

- Relay should not introduce a rival event system.
- Relay should build state/composable helpers that sit on top of GPUI's event and lifecycle APIs.

## Evidence 7: Async work in GPUI is lifetime-aware and cancellation-aware

Representative GPUI async APIs:

- entity-scoped `spawn` in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:235)
- window-scoped `spawn_in` in [context.rs](F:/Workspace/zed/crates/gpui/src/app/context.rs:479)

Tokio bridge evidence:

- `gpui_tokio::init` in [gpui_tokio.rs](F:/Workspace/zed/crates/gpui_tokio/src/gpui_tokio.rs:10)
- task cancellation on GPUI task drop in [gpui_tokio.rs](F:/Workspace/zed/crates/gpui_tokio/src/gpui_tokio.rs:45) and [gpui_tokio.rs](F:/Workspace/zed/crates/gpui_tokio/src/gpui_tokio.rs:66)

Interpretation:

- GPUI async is not a generic detached promise model.
- it is tied to app/window/entity lifetime and carefully bridges foreground/background execution.

Implication for Relay:

- Relay's async abstractions should be query/mutation/resource-oriented, but still lifetime-aware.
- cancellation semantics should compose with GPUI tasks, not sidestep them.

## Evidence 8: GPUI itself documents the split between entities, views, and elements

Primary docs:

- high-level framing in [README.md](F:/Workspace/zed/crates/gpui/README.md)
- contexts in [contexts.md](F:/Workspace/zed/crates/gpui/docs/contexts.md)
- entity ownership and notify/observe model in [_ownership_and_data_flow.rs](F:/Workspace/zed/crates/gpui/src/_ownership_and_data_flow.rs)

Key documented claims from GPUI:

- entities are app-owned durable state;
- views are renderable entities;
- elements are the low-level imperative drawing layer;
- contexts are the service surface for state and lifecycle.

Implication for Relay:

- the most stable Relay design is one that maps onto GPUI's own conceptual model instead of replacing its terminology and boundaries.

## Consolidated Design Pressure

The evidence consistently points to this architecture:

1. GPUI remains the rendering/lifecycle/platform substrate.
2. Relay should focus on:
   - fine-grained state;
   - derivation;
   - resources;
   - selectors;
   - form models;
   - composables for app logic.
3. Relay should not:
   - replace entities;
   - replace GPUI eventing;
   - replace element-local state;
   - assume browser-style reconciliation semantics.

## What This Strengthens In The Previous Design Note

This evidence strengthens the earlier claims that:

- `ReactiveView` should be convenience, not centerpiece;
- entity-scoped composables are a bigger missing opportunity than more runtime indirection;
- element-local state and app/domain state need to stay separate;
- SolidJS is the best reactive influence, but only at the state graph level, not at the DOM/render model level;
- Vue-style composables are a stronger DX model for Relay than React-style universal hooks.
