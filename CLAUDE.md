# CLAUDE.md — Relay UI Kit

## Project Overview

A GPUI-based desktop UI component library for native Rust desktop applications. Four
crates in strict layered architecture: `relay_ui_primitives` → `relay_ui_components` →
`relay_workbench_ui` → `relay_gallery` (binary).

## Build & Test

```bash
cargo check --workspace          # Fast compilation check
cargo test --workspace           # Run all tests (133 tests)
cargo run -p relay_gallery       # Launch interactive gallery
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
```

Rust toolchain: **1.95.0** (pinned in `rust-toolchain.toml`)

## Architecture Rules

### Layer Enforcement
- `Layer::Primitive` — depends only on itself
- `Layer::Component` — depends on Primitives, Components
- `Layer::Workbench` — depends on Primitives, Components, Workbench
- `Layer::Gallery` — depends on everything

Enforced by compile-time tests in `crates/relay_ui_primitives/src/contract/composition.rs`
that parse `Cargo.toml` dependency graphs.

### Component Patterns

1. **All components are `RenderOnce`** — stateless builder structs. The host owns all
   mutable state and re-creates components on state change via `cx.notify()`.
2. **View-free callbacks** — `Box<dyn Fn(...)>` handlers, never `Entity<X>` listeners.
3. **Host-owned state** — Interactive components (`TextInput`, `Checkbox`, `Select`, etc.)
   take state references in `new()`, never own state.
4. **Window-keyed state** — Only `ScrollSurface` uses this, for visual scroll position.

### Contract System (`crates/relay_ui_primitives/src/contract/`)

- `composition.rs` — Layer enum + dependency validation
- `state.rs` — Which components use HostOwned vs WindowKeyed state
- `motion.rs` — Motion duration/direction/policy rules per component
- `event.rs` — Standard event names (`on_click`, `on_select`, etc.)
- `input.rs` — Input value kinds, action kinds, validation states
- `layout.rs` — Border widths, radius scale, scrollbar dimensions, overlay priorities

### Theme (`styles/theme.rs`)

Semantic color tokens via GPUI `Global`. Access via `cx.theme()`. Currently light theme
only; semantic naming enables dark theme drop-in.

### Handler Types (`interaction.rs`)

Centralized type aliases: `ClickHandler`, `SelectHandler`, `ChangeHandler<T>`,
`DismissHandler`, `KeyHandler`, etc. All are `Box<dyn Fn>` or `Rc<dyn Fn>`.

## Adding a New Component

1. Create the file under `crates/relay_ui_primitives/src/components/<category>/`
2. Implement `#[derive(IntoElement)]` struct with builder methods
3. Implement `RenderOnce` — consume `self`, produce `div()` tree
4. Re-export in the category `mod.rs` and crate `lib.rs`
5. Add to relevant contract tables in `contract/` if applicable
6. Add tests in an inline `#[cfg(test)] mod tests` block
