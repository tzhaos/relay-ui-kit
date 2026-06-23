# Relay Redesign Notes

## Goal

Reframe Relay around the real capabilities of GPUI instead of around web-framework imitation.

The target is not:

- "SolidJS for Rust" in a literal sense;
- a browser-style virtual DOM;
- a universal state system that replaces GPUI's own entity and window primitives.

The target is:

- the best developer experience for Rust desktop apps on GPUI;
- fine-grained state where it helps;
- explicit lifetime and ownership where desktop apps require it;
- ergonomic composition that feels modern without fighting GPUI.

## What GPUI Actually Is

After reviewing `F:\Workspace\zed\crates\gpui`, `gpui_macros`, `ui`, and `workspace`, the important realities are:

1. GPUI is entity-first.
   Long-lived state lives in `Entity<T>`, owned by `App`, not inside ephemeral render closures.
2. GPUI is render-tree-ephemeral, not DOM-retained.
   Element trees are rebuilt every frame, but entities and some element-local state are retained.
3. GPUI already has two levels of state.
   Heavy state uses `Entity<T>`; light transient UI state uses `Window::use_state` / `with_element_state`.
4. GPUI rendering is not React reconciliation.
   Invalidations happen through `Context::notify` and entity/window tracking, not through diffing a virtual node graph.
5. GPUI already has an effect cycle.
   `notify`, `emit`, `defer`, entity creation, and window refresh are queued in the app update cycle.
6. GPUI already has a component story.
   `RenderOnce` + `#[derive(IntoElement)]` gives data-only reusable components without forcing every component to be an `Entity`.
7. GPUI performance primitives are explicit.
   `AnyView::cached`, `uniform_list`, custom elements, and `with_element_state` are first-class escape hatches.

These constraints mean Relay should be a GPUI-native augmentation layer, not a competing runtime.

## What To Learn From React, Vue, SolidJS

### From React

Keep:

- component ergonomics and local reasoning;
- explicit derived state instead of implicit mutable coupling;
- devtools-friendly mental models;
- effect hygiene and cleanup discipline.

Do not copy:

- hook-only architecture;
- "everything is a component function";
- render-time closure state as the main unit of ownership.

React's model is optimized for browser component trees and reconciliation.
GPUI is not that.

### From Vue

Keep:

- composables as reusable stateful logic units;
- clear separation between runtime primitives and higher-level app patterns;
- a bias toward declarative async/resource state;
- good defaults for forms, validation, and two-way input flows.

Do not copy:

- magic proxy-driven mutation semantics;
- template/compiler assumptions that do not map cleanly to Rust types.

Vue is strongest as a "developer-experience packaging" reference.
That matters a lot for Relay.

### From SolidJS

Keep:

- fine-grained signals and memos;
- `untrack`-style explicit escape hatches;
- source/effect separation;
- resource semantics that distinguish initial loading from reloading.

Do not copy:

- browser DOM ownership assumptions;
- assuming that every fine-grained dependency should directly own view updates at element granularity.

SolidJS is the closest conceptual influence, but GPUI's cache boundary is the entity, not the DOM node.

## The Most Important Design Correction

Relay should stop thinking of itself as "the reactive runtime for GPUI" and instead become:

"the reactive application-state and composition toolkit for GPUI."

That wording change matters.

It implies:

- GPUI keeps owning rendering and lifecycle truth;
- Relay owns ergonomic state derivation, resource flow, and composition helpers;
- UIKit owns correctness of controls and workstation patterns.

## Recommended Three-Tier Architecture

### Tier 1: GPUI substrate

Owned by GPUI itself:

- `App`, `Context<T>`, `Window`;
- `Entity<T>`;
- `Render` and `RenderOnce`;
- `with_element_state` / `use_state`;
- actions, focus, subscriptions, events;
- view caching and custom elements.

Relay must not wrap these so aggressively that their semantics disappear.

### Tier 2: Relay core

This is where Relay should be strongest:

- `Signal<T>`;
- `Memo<T>` / `derived`;
- `Effect` / `watch` / `watch_changes`;
- `Resource<T, E>`;
- selector and list coordination primitives;
- context injection for reactive cross-layer values;
- form and validation state orchestration;
- async bridge helpers aligned with GPUI lifetime.

This layer should remain UI-agnostic.

### Tier 3: Relay app/composable layer

This is the biggest missing DX opportunity.

Relay should introduce reusable app-facing composition units, similar to Vue composables or React custom hooks, but Rust/GPUI-native:

- `use_selection_model(...)`
- `use_async_query(...)`
- `use_command_palette_state(...)`
- `use_form_model(...)`
- `use_split_view_model(...)`
- `use_focus_ring_state(...)`
- `use_filtered_list(...)`

These should not require a separate magical runtime.
They should just be typed helper structs/functions built on Tier 2.

## What Relay Should Not Do

1. Do not try to replace `Entity<T>`.
   GPUI entities are the actual lifetime boundary.
2. Do not push every state atom into Relay signals.
   `Window::use_state` is better for tiny per-element ephemeral interaction state.
3. Do not hide `Window` and `Context<T>` too much.
   GPUI's focus, overlay, action, and window APIs are too important.
4. Do not overfit to web patterns like hook ordering rules or "render is pure".
   GPUI already has explicit deferred/effect/update semantics.
5. Do not make Relay a second event system.
   GPUI already has `observe`, `subscribe`, `emit`, action dispatch, and focus events.

## Proposed Relay Scope Refinement

### Keep and strengthen

- fine-grained signals;
- memo/derived values;
- resource reloading semantics;
- selector-backed list modeling;
- `untrack`;
- form dirtiness and binding support.

### Reframe

- `ReactiveView`
  Should be positioned as ergonomic sugar only, not a fundamental abstraction.
- `StateScope`
  Useful, but should evolve toward a more composable "entity composition host" rather than a bag of retained handles.
- `Reactive` derive
  Good as ergonomics, but secondary to better composition APIs.

### Add

- composables/modules for app patterns;
- async query/mutation conventions;
- cancellation-aware task helpers;
- optimistic update patterns;
- error boundary / surface-state helpers;
- devtools/debug tracing hooks for reactive graphs;
- test helpers for reactive state assertions.

## A Better State Taxonomy

Relay should document and encode four distinct state classes:

1. Domain state
   Persistent, meaningful app state. Use `Entity<T>` plus Relay signals/resources.
2. View-model state
   Derived and interaction-coordination state. Use Relay heavily here.
3. Element interaction state
   Hover, drag highlight, temporary geometry, local animation toggles. Prefer GPUI element state.
4. Window/workspace state
   Focus, overlays, panes, platform/window integration. Stay close to GPUI.

One of the current risks is treating categories 2 and 3 as the same thing.

## Recommended Composable Model

A Relay composable should usually be one of two shapes:

### Shape A: pure state bundle

Returns a struct of signals, memos, actions, and resources.

Good for:

- filters;
- search queries;
- pagination;
- selection;
- form dirtiness;
- async loading state.

### Shape B: entity-scoped controller

Owns subscriptions/effects tied to a GPUI entity lifetime.

Good for:

- focus coordination;
- source-driven resource loading;
- keyboard navigation models;
- pane/session/task orchestration.

This gives us the ergonomic value of React/Vue custom hooks without pretending Rust has JS closure semantics.

## Async Design Direction

GPUI already provides:

- foreground tasks;
- background executor;
- async contexts;
- `gpui_tokio` integration.

Relay should build a modern async model on top:

- queries with `Pending` / `Reloading` / `Ready` / `Error`;
- invalidation and reload triggers;
- stale-while-revalidate semantics;
- mutation helpers with optimistic updates and rollback support;
- entity-safe task cancellation.

This is where Relay can outperform ad hoc GPUI usage significantly.

## Rendering Design Direction

Relay should embrace GPUI's existing split:

- `RenderOnce` for stateless or lightly stateful reusable components;
- `Render` entities for durable stateful surfaces;
- `AnyView::cached` and retained subviews only where entity retention is actually valuable;
- custom elements only for hotspots or special layout/rendering behavior.

The mistake would be forcing every reusable abstraction into retained `SubView`/entity form.

The recommended rule:

- prefer components first;
- promote to entity-backed subviews only when lifecycle, async, focus, or cache boundaries justify it.

## Suggested API North Star

The best Relay DX would feel like:

- SolidJS in how derived state is expressed;
- Vue in how reusable stateful logic is packaged;
- React in how app structure is understandable;
- GPUI in how rendering, focus, windows, and performance actually work.

In practice, that means the ideal Relay API surface is:

- explicit;
- typed;
- lifetime-aware;
- entity-aware;
- cancellation-aware;
- desktop-oriented rather than browser-oriented.

## Concrete Recommendations For The Next Iteration

1. Demote `ReactiveView` from centerpiece to convenience.
2. Keep `Signal`, `Memo`, `Resource`, `Selector`, `Form` as core primitives.
3. Add a first-class composables layer for stateful app logic.
4. Document when to use GPUI element state vs Relay signals.
5. Add async query/mutation helpers before adding more macro magic.
6. Keep `relay_uikit` focused on correctness and workbench-grade coverage, not on inventing new state semantics.
7. Treat `relay_macros` as optional ergonomics, not architectural foundation.
8. Invest in debug/test tooling for the reactive graph and resource flows.

## Bottom Line

The best Relay is not a clone of a web framework.
It is a GPUI-native state and composition layer that selectively absorbs:

- SolidJS's fine-grained reactivity,
- Vue's composable DX,
- React's mental-model clarity,

while preserving the things GPUI is already uniquely good at:

- entity ownership,
- explicit lifetimes,
- retained view caching,
- high-performance custom rendering,
- desktop/window/focus integration.
