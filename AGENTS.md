# AGENTS.md

Ely GPUI Component. A component library for GPUI, in light and dark.

## Stack

- Rust 1.95, edition 2024. One crate: `ely-gpui-component`.
- gpui 0.2.2 from crates.io. Feature `runtime_shaders` is on by default: machines without Xcode lack the Metal compiler.
- Assets, embedded with `rust-embed`: Lucide 1.48.0 icons (ISC), Inter 4.1 and JetBrains Mono 2.304 (OFL).
- macOS extras (tray icon, Dock badge) call AppKit through `cocoa` 0.26 and `objc` 0.2, the crates gpui already links.
- Gallery: `examples/gallery`. Website: `frontend/` (Vite 8, pnpm), built after the components.

## Commands

- `cargo run --example gallery` opens the gallery. `-- --page <slug>` starts on a page.
- `cargo run --example gallery -- --capture <dir>` writes PNGs of every page, light and dark, top to bottom, then each page's scripted states, including windows the demos open. macOS only.
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
- Files stay at or under 500 lines. Comments are one short line, and rare.
- `motion::duration()` honors reduced motion. Springs overshoot: use them inside animators, never as gpui easings, because gpui asserts eased values stay in `0..=1`.
- `svg()` paints only with its own `text_color`. `Icon` always sets one.
- Buttons stay out of focus on click (`prevent_default` on mouse down). Tab still reaches them.
- The focus ring is a 1px focus-colored border (`FocusRing`). gpui paints shadows under the element, so a shadow ring floods transparent elements. Focusable elements keep a 1px border, transparent at rest.
- `ely_gpui_component::init` binds Tab and Shift-Tab to `FocusNext` and `FocusPrev`. Wrap the root in `FocusScope` and focus its handle.
- gpui's `Window::dispatch_event` returns a private type, so the gallery scripts mouse input by posting `NSEvent`s to its own queue.
- `hover()` needs an element id.
- gpui sends a drag's moves to every listener of its type. Each drag payload carries its owner's `EntityId`, and handlers check it.
- A component that takes a starting value from its owner keeps it in `layout::seeded`: a new value from the owner replaces local drags.
- The capture harness finds windows by gpui handle through `raw-window-handle`, never by scanning the process's windows.
- Posted events cannot move a macOS window, so window drags are not scripted. `drag_region` follows Zed's title bar.
- Set an explicit line height on any box that clips text. gpui's default leading is taller than a tight box, and the clip eats descenders.
- `img()` keeps loading state only with an id. Content masks are rectangles, so rounded corners survive only when the image fills its box without cropping.

## Decisions

- 2026-09-24: official gpui 0.2.2 over the `gpui-pre` snapshot. Older API, first-party publisher.
- 2026-09-24: one crate. Revisit if an incremental check passes 90 seconds.
- 2026-09-24: gpui 0.2.2 has no accessibility tree and no tray, badge, or notification API. Those entries carry `blocked` or `prove`.
- 2026-09-24: the gallery photographs its own window with `CGWindowListCreateImage`. No Screen Recording permission needed.
- 2026-09-24: license MIT OR Apache-2.0.
- 2026-09-25: layouts persist as versioned JSON through serde. Unknown fields are ignored. Newer versions, unknown panels and bad pane trees are refused, and restore is all or nothing.
- 2026-09-25: an entry built from a later chapter's parts lands with that chapter; its line points there. MenuBar goes to Menus, QuickLauncher to Navigation.
- 2026-09-25: SystemNotification is blocked. An unbundled app has no notification center (probed on macOS 27.2), and gpui 0.2.2 has no API.
- 2026-09-25: `unexpected_cfgs` declares `feature = "cargo-clippy"`, which the objc 0.2 macros test.
