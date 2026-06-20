# Relay UI Kit — Project Standards

This document defines the coding, documentation, naming, and workflow conventions for
the Relay UI Kit project. Every contributor is expected to follow these standards.
They are enforced by CI (`cargo fmt --check --all`, `cargo clippy --workspace -- -D warnings`)
and code review.

---

## 1. Comments

### 1.1 Doc comments (`///`, `//!`)

**Hard rules:**

- Every `pub` type (struct, enum, trait) **must** have a `///` doc comment.
- Every `pub` function or method **must** have a `///` doc comment.
- Every crate `lib.rs` **must** open with a `//!` module-level doc comment.
- Every `mod.rs` **must** open with a `//!` module-level doc comment.
- Every variant of a `pub enum` **must** have a `///` comment.

**Style:**

```
/// A single-sentence summary. No trailing period needed.
```

```
/// A multi-paragraph doc comment starts with a one-line summary, then a blank
/// line, then details. Reference other types with [`TypeName`]. Wrap inline
/// code in backticks: `field_name`.
///
/// Code blocks use ```ignore so they are not compiled:
/// ```ignore
/// let btn = Button::new("btn").primary();
/// ```
```

**Module-level template:**

```rust
//! One-line summary of this module's responsibility.
//!
//! Expanded prose: what it provides, where it sits in the architecture,
//! the primary public types and entry points.
```

**Examples from the codebase:**
[`theme.rs`](crates/relay_ui_primitives/src/styles/theme.rs),
[`interaction.rs`](crates/relay_ui_primitives/src/components/interaction.rs),
[`contract/mod.rs`](crates/relay_ui_primitives/src/contract/mod.rs).

### 1.2 Inline comments (`//`)

**Use for:**

- **Section headers** — `// ---------- Title ----------` to separate logical blocks
  within a file. See the `Theme` struct in
  [`theme.rs`](crates/relay_ui_primitives/src/styles/theme.rs) for the canonical
  example.
- **Struct field groups** — A short `// --- Group label` line above related fields.
- **Implementation notes** — Explain **why** something is done, not **what** is
  done (the code already says what).

**Never:**

- Comment out dead code — delete it. Git history can recover it if needed.
- Use `/* */` block comments anywhere in the project.

---

## 2. Naming

### 2.1 Modules and files

| Rule | Example |
|------|---------|
| `snake_case`, **singular** noun | `button`, `choice`, `input`, `overlay` (not `buttons`) |
| Directory uses `mod.rs` style | `overlay/dialog/mod.rs` |
| File name matches module name | `app_shell.rs` → `mod app_shell;` |
| Category directories are nouns | `controls/`, `display/`, `feedback/`, `input/`, `overlay/` |

**Exceptions:** `styles/` and `assets/` are kept as-is (established convention).

### 2.2 Types (struct, enum, trait)

| Rule | Example |
|------|---------|
| `UpperCamelCase` | `Button`, `IconButton`, `SplitPane` |
| Component name = visual role | `Badge`, `Checkbox`, `Tabs`, `AppShell`, `LauncherMenu` |
| Enum suffix: `Variant`, `Style`, `Size`, `Kind` | `ButtonVariant`, `BadgeStyle`, `IconSize`, `EventKind` |
| Trait: adjective or `-Ext` suffix | `ActiveTheme`, `MotionExt` |

### 2.3 Functions and methods

| Rule | Example |
|------|---------|
| `snake_case` | `primary()`, `on_click()`, `min_width()` |
| Constructor is always `new()` | `Button::new("id")`, `Checkbox::new("id", &mut checked)` |
| Builder method name = field name | `fn label(mut self, label: ...) -> Self` |
| Convenience shorthand methods | `primary()` sets `self.variant = ButtonVariant::Primary` |
| Event handlers: `on_<event>` | `on_click()`, `on_select()`, `on_change()`, `on_dismiss()` |

### 2.4 Variables and fields

| Rule | Example |
|------|---------|
| `snake_case`, short and clear | `theme`, `cx`, `handler`, `interactive` |
| Field name matches builder method | `self.label` ↔ `fn label()` |
| Destructure with descriptive names | `let (bg, border, fg) = ...` not `let (a, b, c) = ...` |
| Unused variables: `_` prefix | `_window` in `RenderOnce::render` |

### 2.5 Constants and statics

| Rule | Example |
|------|---------|
| `SCREAMING_SNAKE_CASE` | `SCROLL_GUTTER_WIDTH`, `RADIUS_SM`, `BORDER_WIDTH` |
| Central layout constants in `contract/layout.rs` | `RADIUS_*`, `SCROLL_*`, `OVERLAY_*` |
| Local constants at module top | |

### 2.6 Test functions

| Rule | Example |
|------|---------|
| `snake_case`, descriptive phrase | `progress_ratio_clamps_overflow` |
| Name implies the assertion | `split_state_skips_subpixel_resize` |
| No `test_` prefix needed | Already under `#[test]` |

### 2.7 Crate names

| Rule | Example |
|------|---------|
| `snake_case`, `relay_` prefix | `relay_ui_primitives`, `relay_workbench_ui` |
| Directory drops the prefix | `crates/relay_ui_primitives/` |
| Package name matches directory | `relay_ui_primitives = { path = "crates/relay_ui_primitives" }` |

---

## 3. Documentation

### 3.1 README.md

The project README follows this structure (see [`README.md`](README.md)):

1. **One-line title** — what the project is
2. **Short description** — who it's for, why it exists
3. **Architecture diagram** — ASCII dependency tree
4. **Crate table** — name, purpose, approximate LOC
5. **Quick Start** — a single `cargo run` command
6. **Key Patterns** — 3–5 core design decisions
7. **Development** — essential commands
8. **License**

### 3.2 CLAUDE.md

The AI assistant guide follows this structure (see [`CLAUDE.md`](CLAUDE.md)):

1. **Project Overview** — one sentence + crate hierarchy
2. **Build & Test** — command reference, profile table, directory conventions
3. **Architecture Rules** — layer enforcement, component patterns, contract system
4. **Adding a New Component** — step-by-step checklist
5. **Standards Reference** — pointer to this document

### 3.3 API docs (`cargo doc`)

- Every `pub` item must produce valid `cargo doc` output.
- Cross-reference with `[`Type`]` syntax (Rustdoc auto-links).
- Code blocks: use ` ```ignore ` (not compiled) or ` ```rust ` for trivially
  inline-compilable snippets.
- No mandatory `# Examples` section — for a UI component library, examples are
  visual rather than textual.

### 3.4 New component checklist

When adding a component, update these locations:

1. `///` doc comment on the component struct
2. Category `mod.rs`: `pub mod` declaration + `pub use` re-export
3. Crate `lib.rs`: re-export if the component is part of the public API
4. `contract/` tables in `state.rs`, `motion.rs`, `event.rs`, or `layout.rs`
   (as applicable, depending on the component's contract obligations)
5. `CLAUDE.md` typically does **not** need updating (the generic pattern covers it)

---

## 4. Directory Structure

### 4.1 Project root

```
relay-ui-kit/
  .cargo/config.toml           # Project-level cargo config (rustflags, aliases, target flags)
  .github/workflows/
    ci.yml                     # PR checks (fmt, clippy, check+test)
    release.yml                # Tag-triggered release build → GitHub Release
  crates/
    relay_ui_primitives/       # Layer 1: foundational controls, icons, design tokens, contract
    relay_ui_components/       # Layer 2: layout shell, split panes, tabs, toolbars
    relay_workbench_ui/        # Layer 3: terminal, git, launcher, viewers (product compositions)
    relay_gallery/             # Layer 4 (binary): interactive component showcase
  dist/                        # Distribution artifacts (gitignored) — written only by release workflow
    windows/x86_64/
  script/
    ci-check                   # Run full CI suite locally
    clear-target-dir-if-larger-than
  target/                      # Cargo intermediate artifacts (gitignored)
  Cargo.toml                   # Workspace root (profiles, lints, dependencies)
  Cargo.lock                   # Committed (binary crate in workspace)
  CLAUDE.md                    # AI assistant guide
  README.md                    # Human-readable project documentation
  STANDARDS.md                 # This document
  rust-toolchain.toml          # Pinned Rust toolchain
```

### 4.2 Crate internal structure

```
crates/<crate>/
  assets/                      # Static assets embedded at compile time (SVGs, etc.)
  src/
    lib.rs                     # //! crate doc + module declarations + public re-exports
    <category>/
      mod.rs                   # //! category overview + pub mod + pub use re-exports
      component_a.rs           # One primary type per file
      component_b.rs
      sub_component/           # Expand to directory when a single file exceeds ~300 lines
        mod.rs                 #   or when it needs independent sub-modules (state machine, etc.)
        state.rs               #   State machine for interactive components
        handlers.rs            #   Event handler logic
        geometry.rs            #   Layout calculation
```

### 4.3 Directory responsibilities

| Directory | Contains | Does NOT contain |
|-----------|----------|-----------------|
| `contract/` | Rule tables, const arrays, enums, compile-time validation | Business logic, rendering code |
| `styles/` | `Theme` struct, `Tone` enum, `MotionExt` trait, color/radius/spacing constants | Component code |
| `interaction.rs` | Callback type aliases, `callback_builder!` macro | Component-specific interaction logic |
| `components/<category>/` | One file per component (struct + `impl RenderOnce`) | Cross-component shared state |
| `structure/` | Reusable structural controls (`ScrollSurface`) | Application-level compositions |
| `layout/shell/` | `AppShell`, `SplitPane`, `TitleBar`, `StatusBar` | Primitive controls |
| `workbench/` | Product compositions: terminal, git, launcher, viewers | Reusable generic components |
| `gallery/` | Showcase scenes, sample data | Library code |
| `assets/` | Compile-time embedded resources (SVGs) | Runtime-loaded files |

### 4.4 Module organisation

- **One file, one primary struct.** `button.rs` contains only `Button`.
- **Auxiliary types co-locate.** Enum variants and helper structs live in the same
  file as the primary type they serve.
- **Directory threshold.** When a file exceeds ~300 lines, or needs independent
  sub-modules (like a state machine), expand it into a directory with `mod.rs`.
- **mod.rs format.** Always use the `mod.rs` convention (`foo/mod.rs`).
  Never mix `foo.rs` alongside a `foo/` directory in the same crate.
- **Re-exports.** Category `mod.rs` files use `pub use component_a::*;`
  wildcard re-exports. Crate `lib.rs` uses selective `pub use` of the public API.

---

## 5. Code Organization

### 5.1 Import ordering

Three blocks, separated by blank lines, items alphabetized within each block:

```rust
// ① Standard library
use std::rc::Rc;

// ② External crates
use gpui::{
    ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, div, prelude::FluentBuilder, px,
};

// ③ Workspace crates
use crate::{
    icon::{Icon, IconName},
    interaction::ClickHandler,
    theme::ActiveTheme,
};
```

- Alphabetical order within each block.
- Break into multi-line when a crate contributes more than 3 items.
- **Never** `use crate::*` or `use gpui::*` in implementation files.
  Wildcard imports are allowed **only** in `mod.rs` re-export blocks (`pub use foo::*`).

### 5.2 Struct field order

Fields in `#[derive(IntoElement)]` structs follow this order:

```rust
pub struct SomeComponent {
    // ① Identity
    id: ElementId,
    label: SharedString,

    // ② Configuration
    variant: SomeVariant,
    size: Size,

    // ③ State flags
    disabled: bool,
    active: bool,

    // ④ Callbacks — always Option<HandlerType>, always last
    on_click: Option<ClickHandler>,
    on_select: Option<SelectHandler>,
}
```

### 5.3 Builder methods

```rust
impl SomeComponent {
    /// Constructor — required arguments go here.
    pub fn new(id: impl Into<ElementId>, ...) -> Self { ... }

    /// Builder setter — method name equals field name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Convenience shorthand — encapsulates a common configuration.
    pub fn primary(mut self) -> Self {
        self.variant = SomeVariant::Primary;
        self
    }
}
```

### 5.4 RenderOnce implementation

```rust
impl RenderOnce for SomeComponent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        // ① Compute derived values from self and theme
        let colors = ...;

        // ② Build the div tree
        div()
            .id(self.id)
            .when(condition, |this| ...)
            .when_some(handler, |this, h| ...)
            .child(...)
    }
}
```

### 5.5 Conditional rendering

Always use GPUI's fluent conditional methods:

- `.when(bool, |this| ...)` — boolean condition
- `.when_some(option, |this, val| ...)` — `Option<T>` condition
- `.when(!condition, |this| ...)` — negated condition

**Never** branch at the statement level with `if cond { div()... } else { div()... }`.

### 5.6 Event handlers

```rust
// Wrap at the call site: stop_propagation + invoke the user's handler.
.when_some(self.on_click.filter(|_| self.interactive), |this, handler| {
    this.on_click(move |event, window, cx| {
        handler(event, window, cx);
        cx.stop_propagation();
    })
})
```

- Always `.filter()` handlers against the `disabled`/`interactive` flag.
- For handlers shared across iteration (e.g. Tabs with multiple tabs), use
  `Rc::new(handler)` to clone cheaply.

---

## 6. Testing

### 6.1 Location

- All tests live in `#[cfg(test)] mod tests { ... }` blocks **inline** at the
  bottom of each source file.
- There is **no** separate `tests/` directory in any crate.
- Test helper functions are private `fn`s inside the same `mod tests` block.

### 6.2 Naming

```
fn <subject>_<scenario>_<expected_behavior>()
fn progress_ratio_clamps_overflow()
fn split_state_skips_subpixel_resize()
fn every_icon_resolves_through_asset_source()
```

### 6.3 Minimum coverage per file

Every source file with logic should have tests covering:

1. **Construction** — defaults from `new()` are sensible
2. **State transitions** — key method calls change internal state as expected
3. **Edge cases** — zero, overflow, empty, extreme values handled correctly

### 6.4 Contract tests

Files under `contract/` use **data-driven rule-table tests** (see
[`contract/composition.rs`](crates/relay_ui_primitives/src/contract/composition.rs)):

- Define a `const` array of rule structs.
- A single test iterates the array, asserting each rule.
- New rules → append to array → automatically covered.

---

## 7. Git

### 7.1 Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <imperative description>

type:   feat | fix | refactor | perf | build | docs | test | style | ci
scope:  ui-kit | ui-primitives | ui-components | workbench | gallery
```

Description: imperative mood, lower-case start, no trailing period.

Examples:
```
feat(ui-primitives): add Slider component
fix(ui-kit): resolve focus trap in Modal
refactor(ui-components): extract SplitPane state machine
build(ui-kit): establish cargo profiles and CI pipeline
docs(ui-kit): add project standards document
```

### 7.2 Branches

```
master              # Main branch — always buildable
feature/<name>      # Feature development
fix/<name>          # Bug fixes
```

Worktree branches (created by tooling) use the `worktree-` prefix and are
temporary — merge what's needed, then delete.

### 7.3 Pull requests

- One PR = one thing (a new component, a bug fix, a refactor).
- PR title follows the same format as commit messages.
- All CI checks must pass before merging.

---

## 8. Lints and Formatting

### 8.1 Current lint configuration

Defined in the workspace `Cargo.toml` under `[workspace.lints]`:

| Lint | Level | Rationale |
|------|-------|-----------|
| `unsafe_code` | **deny** | Zero tolerance for unsafe |
| `unused_qualifications` | warn | Keep imports concise |
| `unwrap_used` | warn | Prefer `expect` or meaningful error handling |
| `expect_used` | warn | Scrutinize each `expect` for correctness |
| `dbg_macro` | warn | No debug artifacts left in committed code |
| `todo` | warn | No unfinished code |
| `print_stdout` | warn | Libraries must not write to stdout |

### 8.2 Formatting

- **No `rustfmt.toml`** — Rust default formatting is the standard.
- Toolchain pinned to **1.95.0** via `rust-toolchain.toml`.
- CI runs `cargo fmt --check --all` as its first job.
- CI denies warnings (`RUSTFLAGS="-D warnings"`) — developers can iterate
  locally with warnings, but CI gates on cleanness.

### 8.3 Future lint upgrades

- `missing_docs`: promote from `allow` to `warn` once all existing `pub` items
  have been documented.
- Consider `rust_2018_idioms`, `elided_lifetimes_in_paths`, and
  `clippy::pedantic` (selectively).

---

## 9. Prohibitions

| Prohibited | Use instead |
|-----------|-------------|
| `unsafe` code | No alternative — denied by CI |
| `/* */` block comments | `//` or `///` |
| `use crate::*` / `use gpui::*` in impl files | Explicit imports; `pub use` in `mod.rs` is allowed |
| `if cond { div()... } else { div()... }` | `.when(cond, ...)` / `.when(!cond, ...)` |
| `println!` / `dbg!` in library code | `tracing` / `log` macros |
| Commented-out dead code | Delete it (recoverable from Git) |
| Mixed `foo.rs` + `foo/` module layout | Always `foo/mod.rs` |
| Separate `tests/` directory | Inline `#[cfg(test)] mod tests` |
| Field name ≠ builder method name | `field: X` ↔ `fn field(mut self, v: X) -> Self` |
| `Entity<X>`-coupled callbacks | `Box<dyn Fn>` or `Rc<dyn Fn>` via `interaction.rs` aliases |
