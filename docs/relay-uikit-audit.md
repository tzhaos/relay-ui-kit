# Relay UIKit Productization Audit

Updated: 2026-06-24

## Purpose

This document tracks the actual productization status of `relay_uikit`.
It is not a design wish-list and it is not a gallery marketing page.
It exists to answer four concrete questions with evidence:

1. Which exported UIKit surfaces are already exercised inside the gallery binary?
2. Which public APIs are still under-documented or over-constrained?
3. Which families are strong enough to build product surfaces on today?
4. Which gaps should be fixed before broader migration work begins?

## Evidence Snapshot

The following checks were run against the `relay_v2` branch on 2026-06-24:

- `cargo test -p relay_uikit`
  Result: 269 library tests passed, 15 `relay_gallery` binary tests passed.
- `$env:CARGO_TARGET_DIR='target/build-check'; cargo build --workspace`
  Result: workspace build passed.
- `cargo build -p relay_uikit --bin relay_gallery`
  Result: gallery binary build passed.
- `cargo clippy -p relay_uikit --all-targets --all-features -- -D warnings`
  Result: passed.
- Root export coverage audit over `crates/relay_uikit/src/lib.rs` versus `crates/relay_uikit/src/bin/relay_gallery/**`
  Result: `ALL_ROOT_EXPORTS_REFERENCED_IN_GALLERY_BIN`.
- Gallery coverage metadata derived from live scene modules instead of hardcoded sidebar values
  Result: `Core Kit` 16 groups, `Patterns Kit` 13 groups, `Stress Lab` 7 groups, `Workbench` 4 regions.

Recent relevant commits:

- `8bf2536` `refactor(relay_uikit): relax static component contracts`
- `b4d353c` `refactor(relay_uikit): align actions menu public types`
- `1ad2313` `refactor(relay_uikit): internalize input state-machine contracts`
- `dd5199b` `refactor(relay_uikit): trim dead motion and overlay exports`
- `91609bc` `refactor(relay_uikit): tighten root exports around landed surfaces`

## What "Landed In Gallery" Means Here

`relay_uikit` is not treated as landed merely because a component can compile in isolation.
For this audit, a surface counts as landed when it is exercised somewhere in the gallery binary:

- `Core Kit` for low-level controls and display primitives
- `Patterns Kit` for composite surfaces, shell patterns, overlays, and pickers
- `Stress Lab` for constrained layout, dense content, disabled states, and reconciliation churn
- `Workbench` for app-shaped composition with Relay bindings, selectors, resources, and split panes

This matters because the main product risk in GPUI UI work is rarely "does it render once".
The real risk is whether a component keeps behaving under retained state, focus changes, stale selection, resize, empty data, and long-lived host mutations.

## Documentation Coverage Snapshot

Public API documentation is still the weakest area of the crate.
The current numeric snapshot predates the latest rustdoc rewrite batch and should
be treated as a lower bound, not the final post-batch count.

Breakdown by family:

| Group | Documented | Undocumented | Total |
| --- | ---: | ---: | ---: |
| `components/button.rs` | 0 | 3 | 3 |
| `components/choice` | 0 | 3 | 3 |
| `components/controls` | 1 | 14 | 15 |
| `components/display` | 0 | 9 | 9 |
| `components/feedback` | 0 | 7 | 7 |
| `components/form` | 0 | 4 | 4 |
| `components/icon.rs` | 0 | 4 | 4 |
| `components/input` | 0 | 6 | 6 |
| `components/list` | 0 | 7 | 7 |
| `components/row` | 0 | 2 | 2 |
| `interaction` | 7 | 5 | 12 |
| `patterns/command` | 1 | 6 | 7 |
| `patterns/command_menu.rs` | 1 | 2 | 3 |
| `patterns/diff_view.rs` | 0 | 4 | 4 |
| `patterns/display` | 0 | 1 | 1 |
| `patterns/file_viewer.rs` | 0 | 2 | 2 |
| `patterns/input_composer.rs` | 0 | 1 | 1 |
| `patterns/layout` | 1 | 14 | 15 |
| `patterns/markdown_viewer.rs` | 0 | 1 | 1 |
| `patterns/navigation` | 1 | 1 | 2 |
| `patterns/output_line.rs` | 0 | 2 | 2 |
| `patterns/output_log.rs` | 0 | 1 | 1 |
| `patterns/output_resource.rs` | 1 | 1 | 2 |
| `patterns/output_surface.rs` | 0 | 1 | 1 |
| `patterns/overlay` | 4 | 10 | 14 |
| `patterns/picker` | 2 | 3 | 5 |
| `patterns/quick_action.rs` | 0 | 1 | 1 |
| `patterns/scroll_surface` | 0 | 1 | 1 |
| `patterns/session_row.rs` | 0 | 1 | 1 |
| `patterns/source_view.rs` | 0 | 1 | 1 |
| `patterns/tab_strip.rs` | 0 | 1 | 1 |
| `patterns/tab_toolbar.rs` | 0 | 1 | 1 |
| `patterns/task_row.rs` | 1 | 1 | 2 |

Conclusion:

- gallery landing is far ahead of rustdoc quality;
- the crate can already prove a lot of behavior in tests and scenes;
- the public contract is still too hard to learn by reading the API alone.

## Product-Grade Assessment By Family

### Buttons and Action Primitives

Status: strong enough to use, but still under-documented.

What is already good:

- shared `ButtonLike` behavior centralizes pointer and keyboard activation;
- `Button` and `IconButton` encode disabled, expanded, and toggled semantics;
- gallery and stress scenes exercise normal, icon, disabled, and dense-toolbar usage.

Remaining concerns:

- broader family docs still need richer examples and composition guidance;
- icon-only labeling rules should remain a documented hard requirement across every icon-only surface, not only button primitives.

### Text and Numeric Inputs

Status: functionally strong and improving.

What is already good:

- platform input handlers exist for both single-line and multiline text;
- IME-aware methods are wired through GPUI `InputHandler`;
- `TextInputState` has deep unit coverage for cursor movement, selection, word motion, deletion, composition, and UTF-16 range conversion;
- `NumberInput` now supports less restrictive `key_context` configuration;
- `TextInput`, `SearchField`, `TextArea`, and editable `NumberInput` now stay on the Relay-bound editing path instead of carrying preview-only snapshot constructors and fallback rendering branches;
- `SearchField` now clears its bound value by default and treats `Escape` as a real "clear search" action when content is present, so hosts do not need to wire the basic product behavior by hand;
- `NumberInput` now keeps disabled semantics honest across both stepper buttons and inline editing, and its editable configuration no longer depends on builder call order;
- the latest batch removed unnecessary `&'static str` constraints from text and numeric input public APIs.

Remaining concerns:

- rustdoc has improved for `TextInput` and `TextArea`, but the wider input family still needs more end-to-end usage guidance;
- more gallery scenarios should explicitly stress long-value horizontal scrolling and multiline growth under mixed ASCII/CJK content.

### Choices and Selectors

Status: landed and broadly usable.

What is already good:

- checkboxes, radios, toggles, segmented controls, filter chips, and selects all have binding-oriented APIs;
- select/open semantics are wired through shared `OpenState` and `SelectionSource` adapters;
- `Select` and `ItemPicker` now require explicit open controllers instead of silently creating internal fallback bindings, so open-state ownership is visible at the API boundary;
- `ItemPicker` now defaults to no secondary actions, so generic picker triggers no longer inherit branch-specific behavior unless the host opts in;
- `ItemPicker` now auto-dismisses after selection or action handling by default, so hosts no longer need to manually close common picker flows;
- `ItemPicker` presentation is now host-configurable, so panel title and trigger/row iconography do not hardcode branch semantics into the base picker primitive;
- gallery `Select` and `DropdownMenu` scenes now rely on binding-backed open controllers instead of manual open-state synchronization callbacks;
- patterns scenes exercise select, item picker, command picker, and actions menu compositions.

Remaining concerns:

- public docs still under-explain when to prefer a plain `Binding<T>` over a selector-backed model;
- accessibility semantics should keep being verified as more composite menu and picker work lands.

### Lists, Rows, and Tree Surfaces

Status: solid enough for continued migration.

What is already good:

- `ListItem` centralizes keyboard activation and selection affordances;
- tree/list row families are used both in gallery scenes and the app-shaped workbench demo;
- reconciliation behavior is covered by tests in stress and workbench surfaces.

Remaining concerns:

- `ListItem` is better documented now, but the family still needs broader row/tree examples and guidance;
- navigation semantics and focus expectations need explicit rustdoc examples.

### Overlays, Menus, and Dialogs

Status: landed with meaningful composition coverage.

What is already good:

- anchored overlays, menus, dialogs, confirm dialogs, and dropdown/context/popover surfaces all run inside gallery compositions;
- shared open and dismiss adapters already prevent each overlay from inventing a bespoke state model.
- `Menu` now exposes a reusable action-dismiss contract, so keyboard and pointer activation close bound overlays from the primitive layer instead of forcing each host to hand-roll `set(false)` cleanup.
- `ContextMenu` now follows the same explicit controller shape as the other overlay wrappers instead of rendering as a permanently mounted sample-only surface.
- `DropdownMenu` now uses the same `open_bound` builder vocabulary as the rest of the overlay family instead of a one-off `bound(...)` constructor.
- menu rows that participate in checked/selected state now expose checkbox semantics even when currently unchecked, and submenu rows now surface expanded state to accessibility clients.

Remaining concerns:

- `Dialog` and `ConfirmDialog` are better documented now, but overlay lifecycle rules are still more obvious from code than from docs;
- we should keep adding regression scenarios around focus entry/exit and repeated open-close churn.

### Picker, Command, and Output Surfaces

Status: product-oriented and increasingly coherent.

What is already good:

- output resource helpers preserve previous content during reload and failure;
- command and picker patterns are exercised in realistic project/session/task scenes;
- `OutputSurface` now accepts general `ElementId` values instead of forcing static ids.

Remaining concerns:

- public docs do not yet explain the intended relationship between `OutputLine`, `OutputLog`, `OutputSurface`, and `output_resource_snapshot`;
- command and picker families still need more user-facing API guidance.

### Layout and Shell Patterns

Status: structurally strong, documentation weak.

What is already good:

- split panes, title bar, status bar, pane surfaces, shell composition, and workbench assembly are all live in the gallery binary;
- split pane geometry and resize state have targeted tests;
- `SplitPane` now accepts general `ElementId` values instead of forcing static ids.

Remaining concerns:

- layout family rustdoc is still sparse despite being core migration infrastructure;
- more docs are needed for resize ownership, persisted `SplitPaneState`, and host responsibilities.

## What Was Fixed In The Latest Batch

The latest UIKit cleanup batch focused on input-family productization: removing preview-only constructors, tightening editing ownership around live Relay bindings, and closing real interaction gaps that the gallery had been masking with host glue.

Landed changes:

- `TextInput`, `SearchField`, `TextArea`, and editable `NumberInput` now accept owned `String` key contexts instead of forcing `&'static str`.
- `TextInput`, `SearchField`, and `TextArea` no longer expose host-owned snapshot constructors or carry fallback preview rendering branches.
- `SearchField` now clears its bound `TextInputState` by default, and non-empty bound search queries clear on `Escape` instead of requiring host-owned cleanup code.
- `NumberInput` no longer exposes host-owned numeric or inline-editing constructors; the product path is now Relay-bound value ownership plus optional `input_bound(...)` text editing.
- `NumberInput` editable configuration now survives builder call-order changes, so `focused`, `key_context`, and `on_key` are not silently dropped when they are set before `input_bound(...)`.
- disabled `NumberInput` now blocks both stepper activation and inline text editing instead of only dimming the chrome.
- the core gallery search sample now relies on component-owned clear behavior and uses `.on_clear(...)` only for product side effects.
- `SplitPane` and `OutputSurface` now accept general `ElementId` inputs instead of forcing static string ids.
- `ItemPicker` no longer ships branch-specific default action rows; hosts now opt into secondary actions explicitly.
- `ItemPicker` now closes on selection and action handling by default, and the gallery picker scene relies on that component-owned contract instead of manual host cleanup.
- `ItemPicker` now treats title and iconography as host-owned presentation, while the gallery branch picker opts into branch-specific copy and `GitBranch` icons explicitly.
- gallery settings and pattern scenes now use `open_bound` / `DropdownMenu::bound` directly instead of hand-written open/close bookkeeping.
- `Menu` now owns action-dismiss propagation for nested submenu leaves, and `Select`, `DropdownMenu`, `ContextMenu`, plus the direct anchored-menu gallery sample all consume that shared contract.
- `ContextMenu` now supports `open` / `open_bound` control and the gallery demonstrates a real open-dismiss-action loop instead of a permanently visible mock overlay.
- `Select` and `ItemPicker` no longer synthesize hidden open-state fallback bindings during render, which tightens the controller contract around host-owned overlay state.
- `DropdownMenu` now stores shared open control through `OpenState` and the gallery uses the same `new(...).open_bound(...)` pattern as the other overlay wrappers.
- `MenuItem::checked(false)` now still marks the row as checkable, so select-style menus expose consistent accessibility semantics across both selected and unselected options.
- gallery catalog coverage badges now derive from live scene metadata instead of stale hardcoded counts.
- rustdoc was strengthened for high-frequency public surfaces:
  `Button`, `IconButton`, `TextInput`, `TextArea`, `ListItem`, `Dialog`, `ConfirmDialog`,
  `SectionedList`, `TreeView`, `TreeRow`, `NavRow`, `ItemPicker`, `PickerOption`, `PickerAction`,
  `ActionsMenu`, `Menu`, `MenuItem`, and `Select`.
- dead interaction aliases were removed:
  `SubmitHandler`, `SharedSubmitHandler`, `CancelHandler`, `SharedCancelHandler`, and `ChangeHandler`.
- regression tests were added for the relaxed contracts and generic picker defaults.

This is a product-grade improvement because app-shaped desktop surfaces often need dynamically derived ids and key contexts.
Forcing static strings at the UIKit boundary would otherwise leak artificial constraints into higher-level Relay product code.

## Highest-Priority Remaining Work

1. Raise rustdoc quality for the public component families that are already heavily exercised in gallery.
2. Keep auditing keyboard, focus, and dismissal semantics on composite overlay and picker surfaces.
3. Expand gallery stress scenarios for long text, IME-heavy input, and constrained shell compositions.
4. Continue deleting public aliases and exports that do not carry real product value.
5. Re-run a strict `clippy` pass after the upstream `relay` crate warnings/errors are reduced, because workspace-wide linting is currently blocked outside `relay_uikit`.

## Bottom Line

`relay_uikit` is no longer in a "demo-only" state.
Its base and composite families are already materially exercised inside the gallery binary and backed by a non-trivial test suite.

However, it is not yet "finished" in the product sense.
The biggest remaining deficit is not raw rendering capability.
It is public contract clarity: documentation, family-by-family guidance, and continued elimination of API shapes that encode accidental constraints rather than real UI design intent.
