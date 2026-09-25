# Tasks

Progress truth. Work runs top to bottom. No item is skipped.
Each `T` item ends with a codex review (`gpt-6-astra`, effort `max`, herdr split pane). Three rounds at most.

Per-component lines live in `tasks/`. Tags there:

| Tag | Meaning |
|-----|---------|
| `→ path` | Alias or duplicate. One home, at `path`. |
| `native` | gpui 0.2.2 ships it. The gallery shows the native call. |
| `host` | Ely draws the UI. The host app supplies the engine data. |
| `prove` | Needs a crate or platform API not yet proven with gpui 0.2.2. |
| `blocked` | Cannot be real on gpui 0.2.2. Reason given. |
| `(Txx)` | Built early in task `Txx` as a dependency. |

## Items

- [x] T00 Foundation: package, theme, motion, assets, Icon, Button, gallery shell, docs
- [x] T01 Primitives — `tasks/ch01-10.md`
- [x] T02 Typography
- [x] T03 Layout
- [x] T04 Window & Shell
- [x] T05 Buttons & Actions
- [x] T06a Forms · Text
- [x] T06b Forms · Selection
- [x] T06c Forms · Date & Time
- [x] T06d Forms · Color
- [x] T06e Forms · Files
- [x] T06f Forms · Other
- [x] T06g Forms · Structure
- [x] T07a Navigation · Tabs & paths
- [x] T07b Navigation · Places
- [x] T07c Navigation · Palettes & tour
- [x] T08a Menus · Menu & hosts
- [x] T08b Menus · Bar, search & pie
- [x] T09a Overlays · Popovers & dialogs
- [x] T09b Overlays · Guides & floats
- [ ] T10 Feedback
- [ ] T11a Loading — `tasks/ch11-20.md`
- [ ] T11b Motion
- [ ] T12 Data Display
- [ ] T13 Lists & Trees
- [ ] T14 Tables
- [ ] T15 Charts
- [ ] T16a Finance · Market charts
- [ ] T16b Finance · Technical analysis
- [ ] T16c Finance · Quotes & book
- [ ] T16d Finance · Trading
- [ ] T16e Finance · Markets & assets
- [ ] T17a Editor · Core
- [ ] T17b Editor · Intelligence
- [ ] T17c Editor · Search
- [ ] T17d Editor · Panels
- [ ] T17e Editor · Status items
- [ ] T18 Terminal
- [ ] T19 Git
- [ ] T20 Debug
- [ ] T21a Documents · Editing — `tasks/ch21-30.md`
- [ ] T21b Documents · Reading
- [ ] T21c Documents · Knowledge
- [ ] T22 Collaboration
- [ ] T23a AI Chat · Messages
- [ ] T23b AI Chat · Citations
- [ ] T23c AI Chat · Input
- [ ] T23d AI Chat · Conversations
- [ ] T23e AI Chat · Welcome
- [ ] T24 Agent
- [ ] T25 Generative
- [ ] T26 Media
- [ ] T27 Files
- [ ] T28 Messaging
- [ ] T29 Mail
- [ ] T30 Calendar
- [ ] T31 Project — `tasks/ch31-43.md`
- [ ] T32 Canvas & Design
- [ ] T33 DB & Dev Tools
- [ ] T34 Dashboard
- [ ] T35 Settings
- [ ] T36 Account
- [ ] T37 Onboarding & Help
- [ ] T38 Interaction
- [ ] T39 Theme
- [ ] T40 i18n & a11y
- [ ] T41 Maps
- [ ] T42 Misc
- [ ] T43 Library Tooling
- [ ] T44 Capture: every component, light and dark, plus motion clips
- [ ] T45 Website: `frontend/`, Vite 8, pnpm, light and dark, motion
- [ ] T46 E2E: Playwright against the built site
- [ ] T47 Ship: GitHub repo (public), CI, Pages deploy
- [ ] T48 Acceptance: live URL and MVP checklist

## T00 detail

- [x] git, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE`
- [x] `Cargo.toml`: gpui 0.2.2 with `runtime_shaders`, lints
- [x] Theme: light and dark palettes, high contrast, tokens, `ActiveTheme`, animated switch
- [x] Motion: durations, easings, spring
- [x] Assets: Lucide icons, Inter, JetBrains Mono, `AssetSource`
- [x] Icon, Button, IconButton
- [x] Gallery shell: chapter nav, theme toggle, self-capture
- [x] `AGENTS.md`, `README.md`

## Review log

| Item | Round | Verdict | Notes |
|------|-------|---------|-------|
| T00 | 1 | FAIL | capture swallowed errors; root not focused; raw px sizes; gallery copy; tag conflict |
| T00 | 2 | FAIL | AGENTS.md called `border_1` a rem helper |
| T00 | 3 | PASS | |
| T01 | 1 | FAIL | img loading needs an id; radii lost; focus loop cap; tooltip geometry literals |
| T01 | 2 | FAIL | Cover painted outside the box once the clip was removed |
| T01 | 3 | FAIL | two-line doc comment; fixed after the cap, verified by scan, no fourth review |
| T02 | 1 | FAIL | stale selection; strftime and zone fallbacks; \\text flattening; probes missed targets; ellipsis overflow; u64 precision; caret sizes; LineClamp copy; 500-line wording |
| T02 | 2 | PASS | |
| T03 | 1 | FAIL | restore kept cached split and float state; drags lacked an owner; weights unchecked; pane id overflow broke atomic restore; sticky copy escaped the clip; wheel propagated past the viewport; drawer pull lost the pointer; no top dock; grip sizes raw |
| T03 | 2 | FAIL | closing a pane made NaN or infinite weights; drag ghost text black in dark |
| T03 | 3 | FAIL | even split of a denormal weight underflowed; fixed after the cap by normalizing on close and split, verified by tests and capture, no fourth review |
| T04 | 1 | FAIL | system close skipped TitleBar::on_close; hosted windows had no Tab scope; switcher lost Tab to an ancestor, dropped focus on close, kept a stale index |
| T04 | 2 | FAIL | a removed on_close kept blocking the system close; switcher keys bubbled past it |
| T04 | 3 | PASS | |
| T05 | 1 | FAIL | hold task kept its state alive; a third click wedged the done state; border_1 undid the group seam |
| T05 | 2 | FAIL | AGENTS.md said no window is found by scanning; popups are |
| T05 | 3 | PASS | |
| T06a | 1 | FAIL | twin ids in clear and eye buttons; text fields not Tab stops; IME undo; stepping from a stale value; mask literals re-read as input; undo while disabled; Down past the last line; clamp before rounding; stuck mention dismissal; NaN swallowed by min and max |
| T06a | 2 | FAIL | float error in rounded bounds; composition commit skipped filter and length; a typed leading literal was eaten; PIN select-all collapsed; stepper state read the old value |
| T06a | 3 | FAIL | a mask diff misread a select-all replacement; grid tolerance too wide for large numbers; fixed after the cap by fitting masks inside `TextInput` with the exact edit, and a float-error tolerance; verified by tests and capture, no fourth review |
| T06b | 1 | FAIL | kept highlight and cascade trail went out of bounds; keyboard click reclosed MultiSelect and Cascader; disabled rows committed by keyboard; Combobox hid its owner's value; a disabled chosen radio left no Tab stop; a disabled Rating took arrows |
| T06b | 2 | FAIL | a free Combobox cleared the text it mounted with |
| T06b | 3 | PASS | |
| T06c | 1 | FAIL | range preview reseeded the month; disabled pickers still committed; month buttons left the cursor; hour column scrolled before layout; `*/1` day fields were not wildcards; next(0) ran on; clock fell back to UTC; zone offsets went stale; "never runs" overclaimed |
| T06c | 2 | FAIL | the description of an unstarred full day field ignored the OR rule |
| T06c | 3 | PASS | |
| T06d | 1 | FAIL | a zero-size checker hung the paint loop; a dragged stop lost its grip after passing another; `opacity(1.0)` kept alpha, so opaque pickers stayed see-through; inset thumbs drifted from the drawn gradients; swatch fills and checkers crossed rounded corners; the thumb ring was a literal white |
| T06d | 2 | PASS | |
| T06e | 1 | PASS | |
| T06f | 1 | FAIL | a stroke lost the move that started the drag and its release point; language codes were not searchable though the gallery said so; emoji search compared a lowercase query with mixed-case names |
| T06f | 2 | PASS | |
| T06g | 1 | FAIL | form errors shared one animation id, so a later error skipped its entrance; the demo left links out of its saved snapshot; the demo let a lone @ pass as an email |
| T06g | 2 | FAIL | a field's error still keyed its entrance by the label, so twin labels in one form shared it |
| T06g | 3 | PASS | |
| T07a | 1 | FAIL | editor tabs keyed their scroll on the count of changes: a far tab chosen at mount stayed off screen, and after one change every redraw pulled a manual scroll back |
| T07a | 2 | FAIL | in a hidden or zero-width strip the follow asked for a frame on every render, 126 redraws in 2 s |
| T07a | 3 | FAIL | AGENTS.md said the test platform gives text no width; it gives each character a fixed advance (fixed after the cap) |
| T07b | 1 | FAIL | an open navigation menu kept entry and link indexes past lists that shrank, and disabled links still fired by click or Enter |
| T07b | 2 | PASS | |
| T07c | 1 | FAIL | SearchPalette let a disabled result run on Enter, and the palette tests compiled without test-support |
| T07c | 2 | PASS | |
| T08a | 1 | FAIL | a marked row the owner removed panicked on Enter, the trigger's second click skipped give_back, Enter and Space reached parents on press, the harness never released keys so scripted picks failed, and the menus copy said key hints run rows |
| T08a | 2 | FAIL | Ctrl-Enter and Shift-Space reached keys() on press and picked at once |
| T08a | 3 | PASS | |
| T08b | 1 | FAIL | a shrinking pie kept a mark past its end, a hub press lost focus, pie confirm keys bubbled and modified releases picked, an open dropdown stayed put when its host moved, clearing the filter kept the old mark, and the ring sat half a slice off its hub |
| T08b | 2 | FAIL | the pie's opening right click let a root FocusScope take focus before take_focus recorded it |
| T08b | 3 | PASS | |
| T09a | 1 | FAIL | dialog buttons closed through the owner and skipped give_back, popover and hover panels let presses through, and the popover guessed its height |
| T09a | 2 | FAIL | the prompt's Enter release ran from anywhere in the dialog, so Enter on Cancel submitted and on Submit sent twice |
| T09a | 3 | PASS | |
| T09b | 1 | FAIL | the lightbox photo was cropped, the toolbar kept last frame's anchor, Escape missed a spotlight whose target held focus, a disabled Next dropped focus, and a dead arrow's press closed the lightbox |
| T09b | 2 | FAIL | Enter on a focused Back that returned the tour to its first step removed the button and lost focus, so the arrows stopped |
| T09b | 3 | FAIL | Left from a focused Back also removed it and lost focus; take_focus now takes focus back whenever the focused element leaves the tree (fixed after the cap) |
