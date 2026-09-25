# AGENTS.md

Ely GPUI Component. A component library for GPUI, in light and dark.

## Stack

- Rust 1.95, edition 2024. One crate: `ely-gpui-component`.
- gpui 0.2.2 from crates.io. Feature `runtime_shaders` is on by default: machines without Xcode lack the Metal compiler.
- Assets, embedded with `rust-embed`: Lucide 1.48.0 icons (ISC), Inter 4.1 and JetBrains Mono 2.304 (OFL).
- Catalog data: `emojis` (Unicode emoji), `isolang` (ISO 639, native names), `isocountry` (ISO 3166), `iso_currency` (ISO 4217, also minor units for money).
- macOS extras (tray icon, Dock badge) call AppKit through `cocoa` 0.26 and `objc` 0.2, the crates gpui already links.
- Gallery: `examples/gallery`. Website: `frontend/` (Vite 8, pnpm), built after the components.

## Commands

- `cargo run --example gallery` opens the gallery. `-- --page <slug>` starts on a page.
- `cargo run --example gallery -- --capture <dir>` writes PNGs of every page, light and dark, top to bottom, then each page's scripted states, including windows the demos open. macOS only.
- `cargo test --lib --features test-support`, `cargo clippy --all-targets --features test-support -- -D warnings`, `cargo fmt --check`. `scripts/check.sh` runs them with the house rules.
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
- The capture harness finds gpui windows by handle through `raw-window-handle`. AppKit popups, which gpui does not own, come from the window list by process and level.
- Posted events cannot move a macOS window, so window drags are not scripted. `drag_region` follows Zed's title bar.
- An overlay takes focus with `primitives::take_focus` and returns it with `give_back` on every close path.
- A state change made outside render calls `cx.notify()`, or nothing redraws.
- The macOS share picker draws its content in another process; a capture shows only its frame.
- NSColorSampler runs in another process too. The harness cannot pick or cancel with it; `read_srgb` is tested on a constructed `NSColor` instead.
- Every text field is a `forms::TextInput` entity. Its keys live under the `ElyInput` context, bound in `init`. Wrappers take Up, Down, Enter and Backspace first with `capture_action`.
- `TextInput::bounds_for` reads the last layout. Text changed this frame clamps to the laid-out end until the next paint.
- The harness types with `dispatch_keystroke` through `AnyWindowHandle`, which leaves the root view free to redraw.
- The macOS open panel runs out of process too. Escape does not reach it; the harness sends it `cancel:`.
- gpui's `test-support` swaps its executor: `block` fails on a real dispatcher. It stays behind this crate's `test-support` feature, never on for the gallery.
- A tracked `FocusHandle` is a Tab stop only when built with `.tab_stop(true)`. `tab_index` on the div reaches only handles gpui makes itself. `primitives::tab_stop` keeps one per id.
- Floating lists go below their anchor, or above when only above has room (`forms::options::float`). gpui's own switch keeps the anchor point, so a flipped list would cover its trigger.
- A trigger that opens a list goes through `forms::select::listing`: toggle, keys, blur and popup live there, for Select, TabOverflowMenu and Breadcrumb alike.
- A palette goes through `navigation::palette::Palette`: query field, grouped rows, cursor, keys and focus live there. CommandPalette, QuickOpen, QuickSwitcher, SearchPalette and QuickLauncher only build rows.
- A menu goes through `menus::menu`: rows, submenus, keys, focus and presses outside live there, for DropdownMenu, OverflowMenu, SplitButton, ContextMenu, SearchableMenu and MenuBar. A press on the host's own box is the host's to handle, so a trigger toggles and a right click moves a context menu without a close and reopen in one event.
- gpui focuses the nearest focusable ancestor on mouse down, and apps wrap everything in a `FocusScope`. A press that must leave focus alone calls `prevent_default`, as buttons and context regions do.
- gpui draws a window's first frame inside `open_window`. State the first frame reads, such as `cx.set_menus`, is set before it opens.
- gpui clicks a focused element when Enter or Space is released. An overlay that hands focus back picks on release too; a pick on press returns focus first, and the release clicks the opener again.
- A marker that slides between items (segment thumb, tab line) measures them with `motion::slide` and eases with `glide`.
- A picker hands focus to its popup on open and back to its trigger on pick or Escape; it closes once focus leaves both (`forms::date::picker`).
- Motion for a value change keys its animation on `motion::changes`, so it replays per change and stays still on first paint.
- Thumbs stay inside their component's box: the track is padded by half a thumb (`Slider`, `ColorPicker`, `GradientEditor`).
- gpui's `Hsla::opacity` scales alpha; `alpha` sets it.
- A rounded box does not clip its children. Each layer inside takes the radius itself; `checker` takes one.
- Only gpui can build `ExternalPaths` with paths, so drop rules live in `forms::files::dropped`, where tests reach them.
- gpui runs key bindings before key listeners. A container takes a child's bound key through the action with `capture_action`, as `Form` takes `Submit`.
- `ScrollHandle::scroll_to_item` runs in the container's prepaint before its bounds are stored, so a first-frame call misfires. Call it from a child's prepaint once the handle has bounds, then request one frame (`navigation::editor_tabs`). Never poll layout with `request_animation_frame`.
- The test platform never runs next-frame callbacks, and its text metrics are simplified: each character takes a fixed advance. Tests refresh the window in place of the display link, and click where layout does not hang on text, such as padding and slots.
- An absolute element with no insets sits where it would flow: after the content in a block, inside the padding in a flex box. A canvas that measures its parent pins itself with `.top_0().left_0()`.
- Keyed state lives while its element renders in consecutive frames. An overlay rendered only while open starts fresh each time.
- Long grids scroll in a `uniform_list`. gpui has no nearest scroll, so a key move up scrolls with `Top` and down with `Bottom` (`forms::glyphs`).
- Masks reshape edits inside `TextInput::set_fit`, which sees the replaced range and the typed text. A diff after the fact cannot tell typed characters from kept ones.
- Ids inside a reusable component carry its owner's id or `EntityId`, animation ids too. Twin ids share focus, click and animation state.
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
