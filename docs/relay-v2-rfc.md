# Relay v2 RFC Draft

## Status

Draft

## Motivation

Relay v1 established several useful primitives:

- signals
- memos
- effects
- resources
- selectors
- form aggregation

But after reviewing GPUI and Zed's real architecture, the main missing value is no longer primitive state itself.
The real missing value is a better composition layer for desktop app state and interaction flows.

Relay v2 should therefore evolve from:

"a reactive runtime for GPUI"

to:

"a GPUI-native reactive state and composable architecture toolkit."

## Design Goals

1. Preserve GPUI as the source of truth for rendering, entity ownership, focus, windows, and eventing.
2. Keep Relay primitives explicit and Rust-typed.
3. Add a first-class composables layer for real app/view-model logic.
4. Standardize async query/mutation/resource workflows.
5. Distinguish clearly between:
   - domain state
   - view-model state
   - element-local transient state
   - window/workspace coordination state
6. Improve ergonomics without hiding performance boundaries.

## Non-Goals

1. Replacing `Entity<T>`.
2. Replacing GPUI `Context<T>` / `Window`.
3. Building a virtual DOM.
4. Copying React hooks semantics directly.
5. Routing all local UI interaction state through Relay signals.

## Architectural Layers

### Layer 0: GPUI substrate

Owned by GPUI:

- `App`
- `Context<T>`
- `Window`
- `Entity<T>`
- `Render`
- `RenderOnce`
- `use_state` / `with_element_state`
- actions, focus, events, subscriptions
- view caching
- virtualization and custom elements

Relay v2 must compose with this layer, not obscure it.

### Layer 1: Relay core primitives

Retained and refined from v1:

- `Signal<T>`
- `Memo<T>`
- `Effect`
- `Resource<T, E>`
- `Selector<K>`
- `Form`
- reactive context
- batch / untrack

These remain the foundation.

### Layer 2: Relay composables

New in v2.

This is the main value-add layer.

Examples:

- `use_query(...)`
- `use_mutation(...)`
- `use_single_selection(...)`
- `use_multi_selection(...)`
- `use_filtered_items(...)`
- `use_tree_projection(...)`
- `use_picker_model(...)`
- `use_form_model(...)`
- `use_focus_state(...)`
- `use_entity_subscription(...)`

These should be ordinary Rust APIs built on Layer 1 and GPUI substrate APIs.

### Layer 3: Relay-integrated UIKit patterns

`relay_uikit` should consume the above layers, not define new state semantics.

Its role is:

- correctness of controls
- robust interaction patterns
- comprehensive workbench surfaces

## State Taxonomy

Relay v2 should teach and encode four different state classes.

### 1. Domain state

Examples:

- task/session/workspace data
- durable application settings
- file/tree/search domain models

Recommended representation:

- `Entity<T>` containing Relay signals/resources as needed

### 2. View-model state

Examples:

- filtered command list
- selected item projection
- async loading/reloading state
- validation state
- search result ranking

Recommended representation:

- Relay signals, memos, resources, selectors, composables

### 3. Element-local transient state

Examples:

- hover flags
- drag highlight
- popover geometry
- temporary animation flags

Recommended representation:

- GPUI `use_state` / `with_element_state`

### 4. Window/workspace coordination state

Examples:

- focus
- modal state
- pane activation
- toolbar integration
- platform/window event behavior

Recommended representation:

- GPUI APIs, optionally wrapped in Relay composables

## Core API Direction

### Signals and memos

Keep mostly as-is, but position them more explicitly as view-model primitives rather than universal state storage.

### Resources

Expand `Resource<T, E>` into a more standard query foundation.

Potential additions:

- `Idle`
- metadata such as request generation
- convenience adapters for optimistic updates
- cancellation hooks
- invalidation helpers

### Selectors

Evolve from `Selector<K>` into a broader selection family:

- single selection
- multi selection
- marked selection
- selected item projection
- selection history/reveal helpers

### Forms

Keep `Form`, but add a higher-level form model layer:

- validation state
- touched/dirty/submitted
- async validation support
- reset/commit helpers

## Composable API Direction

Composable APIs should come in two shapes.

### Shape A: pure model composables

Return typed bundles of:

- signals
- memos
- selectors
- resources
- actions

Examples:

- `use_query`
- `use_search_model`
- `use_command_query_history`
- `use_selection_model`

### Shape B: entity-scoped coordination composables

Own effects/subscriptions tied to a GPUI entity.

Examples:

- `use_focus_tracking`
- `use_window_subscription`
- `use_resource_from_source`
- `use_keyboard_navigation`

These can internally use something like a refined `StateScope`, but the public API should read like reusable state modules, not like handle bookkeeping.

## `StateScope` Direction

Current `StateScope` is useful, but too low-level in presentation.

Relay v2 should evolve it into an internal or semi-public lifetime host for composables.

Possible direction:

- `StateScope` remains the effect/resource retention backend
- public APIs expose named composables rather than manual scope pushing

Example:

Instead of:

- create scope
- call `scope.watch(...)`
- call `scope.load_resource_on_changes(...)`

Prefer:

- `let query = use_query_model(...);`
- `let tree = use_tree_projection(...);`
- `let nav = use_keyboard_selection(...);`

## `ReactiveView` Direction

`ReactiveView` should be demoted from architectural centerpiece to convenience helper.

Rationale:

- GPUI already has a clear `Render` model.
- many reusable components are `RenderOnce`, not views.
- the biggest missing value is not render boilerplate elimination.

Recommended position:

- keep `ReactiveView` as sugar for auto-tracked render patterns
- do not build Relay v2 around it

## Async Model

Relay v2 should standardize async state around a query/mutation model aligned with GPUI lifetimes.

### Query goals

- pending vs reloading distinction
- stale result suppression
- cancellation-aware behavior
- entity/window safe updates
- optional blocking fast-paths for warm results

### Mutation goals

- optimistic updates
- rollback hooks
- commit/failure resource states
- follow-up invalidation hooks

Potential APIs:

- `use_query`
- `use_mutation`
- `use_optimistic_mutation`
- `invalidate_query`
- `reload_query`

## Derived Collection APIs

Large app surfaces repeatedly hand-roll:

- filtered arrays
- ranked arrays
- selected item projections
- visible tree projections

Relay v2 should add collection-oriented helpers:

- `derive_vec`
- `filter_vec`
- `ranked_vec`
- `selected_item`
- `tree_projection`

These do not have to be magical.
They just need to make common state projections standard and testable.

## Testing and Tooling

Relay v2 should add developer tooling value, not just runtime primitives.

Recommended areas:

- reactive graph tracing
- resource lifecycle tracing
- query state snapshots
- test helpers for resource/selection/form assertions

This is especially important because desktop workflows are rich in async and focus interactions.

## Migration Strategy

### Phase 1: preserve v1 primitives

Keep:

- `Signal`
- `Memo`
- `Effect`
- `Resource`
- `Selector`
- `Form`

This avoids breaking current experiments.

### Phase 2: introduce composables experimentally

Add a new module namespace for composables.

Examples:

- `relay::composables::query`
- `relay::composables::selection`
- `relay::composables::form`
- `relay::composables::focus`

### Phase 3: move examples and gallery surfaces onto composables

Use:

- command picker
- session/task surfaces
- resource demos
- settings form demos

as migration proving grounds.

### Phase 4: reduce emphasis on lower-level boilerplate helpers

As composables mature:

- reduce conceptual emphasis on `ReactiveView`
- reduce need for manual `StateScope` choreography in app code

## Example Target Surfaces

Relay v2 should be validated against these classes of UI:

1. command palette / picker
2. tree panel with selection and expansion
3. settings form with validation and dirty state
4. async output/log/detail inspector
5. split-pane workbench shell

If the API is awkward for these, it is not ready.

## Summary

Relay v2 should be:

- more GPUI-native than Relay v1 in architecture;
- more SolidJS-like in fine-grained derivation;
- more Vue-like in reusable stateful composition;
- more useful for desktop app developers than a direct web-framework transplant.

The central move is simple:

keep primitives,
add composables,
respect GPUI,
standardize async and selection patterns,
and optimize for the workflows real desktop surfaces actually contain.
