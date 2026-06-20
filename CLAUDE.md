# CLAUDE.md — Relay UI Kit

## Project Overview

A GPUI-based desktop UI component library for native Rust desktop applications. Four
crates in strict layered architecture: `relay_ui_primitives` → `relay_ui_components` →
`relay_workbench_ui` → `relay_gallery` (binary).

## Build & Test

Rust toolchain: **1.95.0** (pinned in `rust-toolchain.toml`)

### Quick commands

```bash
cargo run -p relay_gallery       # Launch interactive gallery (debug)
cargo check --workspace          # Fast compilation check
cargo test --workspace           # Run all tests
cargo fmt --check --all          # Format check
cargo clippy --workspace -- -D warnings
```

### CI commands (use `ci` profile — no incremental, warnings denied)

```bash
cargo ci-check                   # cargo check --profile ci --workspace
cargo ci-test                    # cargo test --profile ci --workspace
cargo ci-clippy                  # cargo clippy --profile ci --workspace -- -D warnings
./script/ci-check                # Run full CI suite locally (fmt → clippy → check → test)
```

### Release build

```bash
cargo build-release              # Optimized binary → target/release/relay_gallery.exe
```

### Maintenance

```bash
./script/clear-target-dir-if-larger-than [MAX_GB] [SOFT_GB]
# Cleans target/ when it exceeds MAX_GB (default 10); soft-cleans workspace crates
# above SOFT_GB (default 5). Safe to run anytime.
```

### Directory conventions

| Directory | Purpose | Lifecycle |
|-----------|---------|-----------|
| `target/` | Cargo-managed intermediate build artifacts | `cargo clean` any time |
| `dist/`   | Final distribution artifacts (release builds) | Decoupled from `target/` |
| `script/` | Build/maintenance helper scripts | Checked in |

### Build profiles (defined in root `Cargo.toml`)

| Profile | Use case |
|---------|----------|
| `dev` (default) | Fast iteration: incremental, `debug=limited`, split-debuginfo |
| `ci` | CI pipelines: inherits dev, `incremental=false` |
| `release` | Distribution: ThinLTO, `codegen-units=1`, stripped, `panic=abort` |
| `release-fast` | Local perf testing: inherits release, no LTO, 16 codegen units |

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

## Standards

All code, documentation, naming, testing, and Git conventions are defined in
**[`STANDARDS.md`](STANDARDS.md)** — the single authoritative source for project
standards. Covers:

- **Comments** — `///` / `//!` rules, `//` section headers, no `/* */` blocks
- **Naming** — modules (singular), types, functions, variables, constants, tests, crates
- **Documentation** — README structure, CLAUDE.md layout, API docs, new component checklist
- **Directory structure** — every directory's responsibilities, module layout, re-export rules
- **Code organization** — import ordering, struct field order, builder methods, RenderOnce
- **Testing** — inline `#[cfg(test)] mod tests`, naming, minimum coverage, contract tests
- **Git** — Conventional Commits, branch naming, PR rules
- **Lints & formatting** — rationale for each lint level, future upgrades
- **Prohibitions** — banned patterns and their alternatives
