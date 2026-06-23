# Relay UIKit Product-Grade Contract

## Mission

`relay_uikit` is not a sketchbook of GPUI examples.
It is Relay's production UI surface for desktop applications, which means every exported
component and pattern must be good enough to survive real workbench usage:

- keyboard-first navigation;
- retained state across rerenders;
- overlay churn and rapid repeated input;
- async loading and failure states;
- accessibility semantics;
- long-lived, evolving host state wired through Relay primitives.

The bar is not "visually plausible".
The bar is "safe to build product surfaces on top of without rewriting the primitive later".

## Product-Grade Means Passing Every Gate

### 1. State Model Gate

A component is not product-grade unless its state ownership story is explicit.

Depending on its role, it must support the right combination of:

- host-owned snapshot + callback usage;
- relay-bound usage via `Binding<T>` / `SelectionSource<T>` / `OpenState>`;
- stable behavior when the host updates state externally;
- no hidden second state system that drifts from Relay or GPUI focus state.

### 2. Interaction Gate

Pointer and keyboard interactions must be equivalent where the component is actionable.

Required behaviors include, as applicable:

- click, press, and repeat-safe interaction;
- `Enter` / `Space` parity for buttons, rows, and menu items;
- arrow-key behavior for selection and navigation surfaces;
- `Escape` dismiss/cancel behavior for overlays and text entry;
- disabled state that blocks both pointer and keyboard paths.

### 3. Focus Gate

Desktop UI quality lives or dies on focus behavior.

Components must define:

- whether they are tabbable;
- what visual focus affordance they expose;
- what happens when nested actionable regions exist;
- how focus moves when overlays open or close;
- whether focus restoration is the host's job or the component's job.

### 4. Accessibility Gate

Product-grade means semantics, not just paint.

Components should supply the correct GPUI role and state metadata where applicable:

- `role`;
- `aria_label`;
- `aria_selected`;
- `aria_expanded`;
- `aria_toggled`;
- any other semantic state the host cannot infer from visuals alone.

If an icon-only or highly compressed affordance cannot be understood without extra labeling,
the API must make that labeling practical instead of optional-by-accident.

### 5. Resilience Gate

The component must behave sanely under hostile but ordinary product conditions:

- empty values and placeholder states;
- long labels and constrained width;
- selection or open state becoming stale after host mutation;
- repeated open/close cycles;
- list reconciliation and identity reuse;
- IME, composition, and non-ASCII text entry where text is involved.

### 6. Verification Gate

A component is not done when it "seems to work".
It is done when its critical invariants are encoded in tests and gallery scenarios.

At minimum, each family should accumulate:

- focused unit tests for state and behavior invariants;
- gallery coverage for interactive/manual verification;
- docs that explain the intended ownership and interaction model.

### 7. Composition Gate

Components and patterns must remain composable.

Good signs:

- host callbacks stay plain and type-erased where useful;
- Relay-owned models are adapted rather than hidden;
- leaf controls do not take unnecessary ownership of product workflow state.

Bad signs:

- component-local booleans trying to model workflow state;
- bespoke per-component controller models with no common vocabulary;
- APIs that only work inside the gallery.

## Family-by-Family Acceptance Matrix

The crate is only "product-grade" when each family reaches its own completion bar.

### Buttons and Action Primitives

Includes `Button`, `IconButton`, and the internal `ButtonLike`.

Must guarantee:

- pointer and keyboard activation parity;
- clear primary/secondary/danger/ghost affordances;
- disabled semantics;
- focus-visible affordance;
- labeling for icon-only actions;
- stable sizing in dense toolbars and forms.

### Text and Numeric Inputs

Includes `TextInput`, `SearchField`, `TextArea`, `NumberInput`, and shared input state.

Must guarantee:

- host-owned and relay-bound editing flows;
- selection, caret, composition, and IME correctness;
- keyboard editing/navigation parity;
- validation and partial-value handling where numeric parsing is involved;
- bounded layout under long content;
- platform text input integration instead of a fake desktop text model.

### Choices and Selectors

Includes `Checkbox`, `Radio`, `Toggle`, `SegmentedControl`, `FilterChip`, and `Select`.

Must guarantee:

- explicit selected/toggled/open semantics;
- keyboard operation and focus movement;
- correct mapping to Relay selection/open state models;
- stable behavior when the selected item disappears or changes externally.

### Lists, Rows, and Tree-Like Surfaces

Includes `ListItem`, `SectionedList`, `TreeView`, `NavRow`, `TreeRow`, `SessionRow`, and `TaskRow`.

Must guarantee:

- row identity and retained subview correctness;
- selected/focused/hovered states that do not fight each other;
- keyboard activation and selection semantics;
- empty, filtered, and mutating collection resilience;
- correct roles and labels for navigational and hierarchical content.

### Overlays and Menus

Includes `Dialog`, `ConfirmDialog`, `Popover`, `DropdownMenu`, `ContextMenu`, `Menu`, and `AnchoredOverlay`.

Must guarantee:

- consistent open/dismiss contract;
- escape/outside-click behavior;
- open-state control via host or Relay binding;
- focus entry and exit rules;
- no accidental stuck-open or stale-anchor states.

### Picker and Command Surfaces

Includes `CommandPalette`, `CommandMenu`, `ItemPicker`, `ActionsMenu`, and related rows.

Must guarantee:

- list reconciliation under changing data;
- selection/open state correctness;
- command keyboard affordances;
- empty/loading/error and async refresh resilience where applicable.

### Feedback and Display

Includes badges, labels, banners, callouts, progress, skeletons, toasts, and empty states.

Must guarantee:

- visual consistency with theme/tone;
- safe behavior in constrained layouts;
- no state semantics hidden in presentation-only APIs.

### Layout and Shell Patterns

Includes `AppShell`, `Pane`, `SplitPane`, toolbars, title bar, status bar, and scroll surfaces.

Must guarantee:

- resize and reconciliation correctness;
- keyboard and focus stability across nested panes;
- correct separation between shell state and leaf component state;
- no geometry glitches under rapid resize or pane switching.

## Relationship to Relay

`relay_uikit` should consume Relay primitives, not reinvent them.

Good directions:

- `Binding<T>` for ordinary two-way controls;
- `SelectionSource<T>` / `SelectionBinding` for keyed selection;
- `OpenState` for shared overlay semantics;
- retained child state through Relay/GPUI entity patterns where identity matters.

Bad directions:

- a second hidden state layer that competes with Relay;
- component APIs coupled to app-specific domain entities;
- visual widgets that bury controller logic the host needs to reason about.

## Documentation and Test Rules

Every public family should converge on the same documentation shape:

1. Crate/module docs explain ownership and composition rules.
2. Public component docs explain what the component is for.
3. Tests encode behavior that product code depends on.
4. Gallery scenes exercise the component in realistic compositions, not just isolated happy paths.

## Current Rollout Order

The practical rollout order should stay bottom-up:

1. action primitives and text entry;
2. choice/selection surfaces;
3. list rows and hierarchical navigation;
4. overlays and menus;
5. picker/command patterns;
6. shell/layout composites;
7. visual-only display and feedback polish.

This keeps us from polishing high-level patterns on top of shaky base primitives.
