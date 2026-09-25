# AGENTS.md

Ely GPUI Component. A component library for GPUI, in light and dark.

## Stack

- Rust 1.95, edition 2024. One crate: `ely-gpui-component`.
- gpui 0.2.2 from crates.io. Feature `runtime_shaders` is on by default: machines without Xcode lack the Metal compiler.
- Assets, embedded with `rust-embed`: Lucide 1.48.0 icons (ISC), Inter 4.1 and JetBrains Mono 2.304 (OFL).
- Gallery: `examples/gallery`. Website: `frontend/` (Vite 8, pnpm), built after the components.

## Commands

- `cargo run --example gallery` opens the gallery. `-- --page <slug>` starts on a page.
- `cargo run --example gallery -- --capture <dir>` writes PNGs of every page, light and dark, top to bottom. macOS only.
- `cargo test --lib`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- `rm -rf target/debug/incremental` after each task item keeps the disk lean.
- `scripts/icons.sh <lucide-name>...` adds icons. Then add a line to `IconName` in `src/primitives/icon.rs`.

## Layout

- `src/<chapter>/`: one folder per chapter of `gpui-components.md`.
- `src/theme`: palettes, tokens, `ActiveTheme`. `src/motion`: durations, easings, spring.
- `examples/gallery/pages/<chapter>.rs`: one page per chapter, in PRD order.
- `TASKS.md` and `tasks/`: progress. Every PRD entry has a line and a tag.

## Rules

- Colors, control and icon sizes, text sizes, radii, shadows come from `cx.theme()`. No raw `px()` literals inside components.
- Gaps and padding use gpui's rem scale helpers (`gap_2`, `px_3`). That scale is the spacing system.
- Borders use gpui's fixed-pixel helpers (`border_1` is 1px). Hairlines do not scale.
- One component, one home. Duplicates point to it.
- Fail fast. No silent fallbacks. Log state changes with `log`.
- Files stay under 500 lines. Comments are one short line, and rare.
- `motion::duration()` honors reduced motion. Springs overshoot: use them inside animators, never as gpui easings, because gpui asserts eased values stay in `0..=1`.
- `svg()` paints only with its own `text_color`. `Icon` always sets one.
- Buttons stay out of focus on click (`prevent_default` on mouse down). Tab still reaches them.
- `hover()` needs an element id.

## Decisions

- 2026-09-24: official gpui 0.2.2 over the `gpui-pre` snapshot. Older API, first-party publisher.
- 2026-09-24: one crate. Revisit if an incremental check passes 90 seconds.
- 2026-09-24: gpui 0.2.2 has no accessibility tree and no tray, badge, or notification API. Those entries carry `blocked` or `prove`.
- 2026-09-24: the gallery photographs its own window with `CGWindowListCreateImage`. No Screen Recording permission needed.
- 2026-09-24: license MIT OR Apache-2.0.
