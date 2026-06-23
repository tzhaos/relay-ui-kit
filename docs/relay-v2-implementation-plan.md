# Relay v2 Implementation Plan

## Goal

Evolve Relay into a GPUI-native reactive state and composable architecture toolkit.

This means:

- keep GPUI as the rendering, focus, action, lifecycle, and window substrate;
- keep Relay core primitives small and explicit;
- add a higher-level composables layer for real desktop app view-model logic;
- let `relay_uikit` consume these abstractions rather than inventing new state semantics.

## Principles

1. Do not replace GPUI.
2. Do not introduce a second UI runtime.
3. Prefer thin, typed composition over hidden framework magic.
4. Optimize for correctness, entity lifetime safety, and debuggability.
5. Use composables to absorb repetitive app-model logic, not to hide GPUI concepts.

## Landed in this branch

First v2 composables are now implemented in `crates/relay/src/composables/`:

- `use_query` / `use_ready_query` / `use_error_query`
- `use_query_from_source`
- `use_mutation`
- `SelectionModel` via `use_selection_model`
- `FormModel` via `use_form_model`
- `FocusState` via `use_focus_state`

These are intentionally modest first steps:

- `Query` standardizes loading, reloading, error, and latest-value status around `Resource<T, E>`.
- `SourceQuery` standardizes entity-scoped source tracking plus initial load / reload behavior.
- `Mutation` standardizes write-side async state, stale completion suppression, retained last-success state, and follow-up hooks.
- `SelectionModel` packages `Selector<K>` with reusable selection-presence and item-projection helpers.
- `FormModel` removes the awkward lifetime split between `Form` and its dirty memo, and adds submitted-state semantics.
- `FocusState` keeps GPUI `FocusHandle` as the source of truth while making focus a composable signal-backed state source.

## Phase 1: stabilize the composables base

Status: in progress

Targets:

- validate naming and API ergonomics on real example surfaces;
- keep the public API small while we learn from integration;
- ensure every composable has focused tests around lifetime and edge cases.

Exit criteria:

- examples or internal surfaces use these APIs directly;
- no need for ad hoc `_effect`, `std::mem::forget(form)`, or repeated loading flags in those call sites.

## Phase 2: source-driven async composition

Status: started

Targets:

- add source-bound query helpers that wrap the existing `watch` / `watch_changes` patterns;
- formalize reload policies and stale-request handling;
- introduce first mutation primitives.

Progress so far:

- `use_query_from_source(...)` is landed.
- `use_mutation(...)` is landed.
- next highest-value additions are query invalidation/reload helpers and optimistic mutation support.

Candidates:

- `use_query_from_source(...)`
- `use_reload_on(...)`
- `use_mutation(...)`
- optimistic update helpers

Why:

This is where Zed-like async controller glue becomes reusable composable logic instead of being rewritten per surface.

## Phase 3: richer selection and projection models

Targets:

- multi-selection
- marked selection
- keyboard navigation helpers
- reveal/scroll policies
- tree projection helpers

Candidates:

- `use_multi_selection(...)`
- `use_marked_selection(...)`
- `use_keyboard_selection(...)`
- `use_tree_projection(...)`

Why:

The evidence from picker-, project-panel-, and tree-like surfaces says selection logic is a major source of repetition and subtle bugs.

## Phase 4: UIKit integration

Targets for `relay_uikit`:

- consume `Query`, `SelectionModel`, `FormModel`, and later mutation/tree helpers;
- keep controls correct and robust;
- make complex workbench patterns easier to assemble without weakening behavior guarantees.

Expected result:

- UIKit components become thinner and more predictable;
- app-specific logic moves into typed composables instead of leaking into controls.

## Phase 5: example and surface migration

Suggested migration order:

1. picker-like example surfaces
2. form-heavy examples
3. async search/query examples
4. tree/panel style surfaces

Migration rule:

Every migration should reduce controller glue or lifetime bookkeeping. If a composable does not clearly improve the surface, we should refine or remove it.

## Explicit non-goals

- replacing GPUI `Entity<T>`, `Window`, or focus APIs
- hiding `Context<T>` everywhere
- recreating React hooks semantics inside Rust
- making `relay_macros` a foundation for architecture

`relay_macros` remains optional ergonomics, not the center of v2.

## Success criteria

We should feel the redesign working when:

- async surfaces stop rewriting loading/reloading/error state by hand;
- selection and form behaviors are standardized instead of re-derived;
- focus and lifecycle coordination become easier to compose safely;
- `relay_uikit` code gets smaller, more correct, and less stateful internally;
- new desktop surfaces read like app-model composition instead of controller soup.
