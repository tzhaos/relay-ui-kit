# Relay Architecture

## Positioning

`relay` is a GPUI-native reactive runtime for stateful desktop and workstation-style applications.
It brings a SolidJS-like reactive authoring model to GPUI without forcing a virtual DOM, scheduler fork, or browser-first lifecycle.

The core design choice is:

- keep GPUI as the rendering, layout, event, and entity-lifecycle engine;
- let `relay` own dependency tracking, invalidation, and higher-level state primitives;
- let `relay_uikit` focus on correct, resilient, reusable controls and interaction patterns.

This gives the project a clean three-layer split:

1. `relay`
   Reactive runtime and application-state primitives.
2. `relay_uikit`
   GPUI components, patterns, layout shells, and gallery verification surfaces.
3. `relay_macros`
   Compile-time ergonomics for field-level reactive state.

## Why Relay Exists

GPUI already provides efficient retained rendering and entity caching, but application authors still need a disciplined state model:

- fine-grained invalidation;
- derived state;
- async resource orchestration;
- keyed selection and retained child views;
- form dirtiness and two-way binding;
- cross-layer context propagation.

`relay` provides those pieces while preserving GPUI's real execution model:

- rendering is still centered on GPUI `Entity` boundaries;
- writes still notify via `cx.notify(...)`;
- app state remains UI-thread-first by default;
- view retention remains explicit through `SubView` and `KeyedSubViews`.

## SolidJS x GPUI: The Fusion Point

The most interesting architectural direction in this repo is not “port SolidJS to Rust”.
It is selectively fusing SolidJS's reactive semantics with GPUI's retained desktop UI model.

### Borrowed from SolidJS

- Signals as the primitive state cell.
- Memos/derived values as cached computations.
- Effects as dependency-tracked reactions.
- `untrack` for snapshot reads without subscription.
- Resource-style async state with loading/reloading semantics.

### Preserved from GPUI

- Entity-driven lifecycle.
- Explicit render functions.
- Retained subviews instead of DOM diffing.
- Native event flow and cached view reuse.
- Desktop/workbench-oriented layout and interaction constraints.

### The result

Relay behaves like a desktop-first reactive kernel:

- SolidJS contributes the dependency graph mental model.
- GPUI contributes the retained rendering substrate.
- Relay adds the glue that maps signal reads to entity invalidation rather than DOM reconciliation.

That is the real innovation boundary of this workspace.

## Core Runtime Model

The runtime in `crates/relay/src/runtime.rs` tracks which observer reads which signal.
Observers are one of:

- GPUI entities rendered inside `track(...)`;
- Relay effects created through `effect(...)` or entity-scoped helpers.

When a signal changes:

1. relay looks up subscribed observers;
2. entity observers are invalidated through `cx.notify(entity_id)`;
3. effect observers are rerun through the relay scheduler;
4. batched writes defer notifications until the outer batch exits.

This keeps the implementation small and aligned with GPUI instead of introducing a second UI scheduler.

## Public Primitive Set

`relay` currently exposes a coherent application-state toolkit:

- `Signal<T>` and `Binding<T>` for mutable state;
- `Memo<T>` and `derived(...)` for cached derivation;
- `Effect`, `watch`, `watch_changes`, and cleanup-scoped effects;
- `Resource<T, E>` for pending/reloading/ready/error flows;
- `Selector<K>` and `SelectedItemExt` for keyed selection;
- `SignalVecExt` for list mutation with correct notification batching;
- `Form` and `StateScope::form()` for dirty-check workflows;
- `provide_context` / `use_context` for cross-layer reactive sharing;
- `SubView` and `KeyedSubViews` for retained entity composition.

This is a good boundary: runtime concerns stay in `relay`, while visual semantics stay out.

## relay_uikit Responsibilities

`relay_uikit` should stay opinionated about implementation quality rather than runtime invention.
Its priority order should be:

1. correctness
   Controls must update state, focus, selection, overlays, and async presentation reliably.
2. robustness
   Edge cases such as empty states, reloading states, keyboard navigation, stale selection, and retained branch switching must not drift.
3. completeness
   A workbench UI needs not only atoms, but panels, pickers, lists, shells, overlays, diff/log/markdown viewers, and gallery stress surfaces.

This separation is healthy:

- `relay` defines state semantics;
- `relay_uikit` proves those semantics survive real interaction surfaces.

## Why `relay_macros` Still Matters

`relay_macros` currently exists for one public capability: `#[derive(Reactive)]`.
That means it is not runtime-critical, but it is API-critical.

Without `relay_macros`, users lose:

- generated `ReactiveFoo` wrappers;
- field-level accessors for signal-backed structs;
- nested reactive field composition through `#[reactive(nested)]`;
- the simplest “plain struct -> reactive state” onboarding path.

Today, it is justified because:

- `relay` re-exports `Reactive` as part of the public API;
- there is a dedicated integration test in `crates/relay/tests/reactive_derive.rs`;
- there is a dedicated example in `crates/relay/examples/reactive_struct.rs`.

So the crate is not redundant yet.
It becomes removable only if the project intentionally decides that:

- field-level derive ergonomics are not part of Relay's product surface; or
- the macro is folded into `relay` and the crate boundary is removed for packaging simplicity.

Until then, `relay_macros` is small but meaningful.

## Near-Term Documentation Direction

The repo should prefer durable design docs over duplicated README prose.
Recommended documentation shape:

- `docs/relay-architecture.md`
  Source of truth for runtime architecture and design intent.
- `docs/relay-uikit-guidelines.md`
  Source of truth for component quality standards and surface coverage.
- crate-level `//!` docs
  Fast API entry points that stay close to code and examples.

That keeps architectural context stable while letting rustdoc explain actual API usage.
