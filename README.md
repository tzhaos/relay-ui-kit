# Relay UI Kit

A **GPUI-based desktop UI component library** built on [Zed's GPU-accelerated Rust UI
framework](https://github.com/zed-industries/zed). Designed for dense, native desktop
applications — terminals, workbenches, launchers, and developer tools.

## Architecture

The project is structured as four crates in a **strict layered dependency hierarchy**:

```
relay_gallery (bin)           — Interactive component showcase app
  └─ relay_workbench_ui (lib) — Product-level workbench compositions
       └─ relay_ui_components (lib) — Layout shell, split panes, tabs
            └─ relay_ui_primitives (lib) — Foundational controls & design tokens
```

Each layer may only depend on itself and lower layers. This is enforced at compile time
by tests in the `contract` module.

| Crate | Purpose | LOC |
|---|---|---|
| `relay_ui_primitives` | ~50 controls, icons, input models, design tokens, interaction types | ~9,400 |
| `relay_ui_components` | App shell, split panes, tabs, title bar, status bar | ~1,700 |
| `relay_workbench_ui` | Terminal UI, git branch, launcher, file/code/diff viewers | ~2,400 |
| `relay_gallery` | Standalone interactive showcase with 7 scene pages | ~3,900 |

## Quick Start

```bash
# Requirements: Rust 1.95.0+
cargo run -p relay_gallery
```

Opens a 1440×900 window with client-side decorations, showcasing all components across
seven gallery scenes: Workbench, Terminal Hub, Review Desk, Command Center, Settings,
Foundation Lab, and Stress Lab.

## Key Patterns

- **RenderOnce components** — All components are builder-pattern structs that implement
  `RenderOnce`. The host view owns all mutable state; components are stateless renderers.
- **View-free callbacks** — Components accept `Box<dyn Fn>` closures rather than
  `Entity<X>`-coupled listeners. This keeps the library decoupled from any concrete app.
- **Contract system** — The `contract` module defines layout constants, state ownership
  rules, motion policies, event naming conventions, and interaction contracts as data
  tables.
- **Semantic theming** — Color tokens are named by intent (`accent`, `danger`,
  `text_secondary`) rather than by role, making a dark theme a drop-in swap.

## Development

```bash
# Check compilation
cargo check --workspace

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --check --all
```

## License

Apache-2.0
