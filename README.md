# Ely GPUI Component

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Status: early. `TASKS.md` tracks every component, chapter by chapter.

## Use

```toml
[dependencies]
ely-gpui-component = { git = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Components" }
gpui = "0.2.2"
```

```rust
use ely_gpui_component::{Assets, theme::{Mode, Theme}};
use gpui::{App, Application};

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        ely_gpui_component::init(cx);
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`init` registers the fonts and the theme. It panics if `Assets` is not wired into the application.

## Gallery

```sh
cargo run --example gallery
cargo run --example gallery -- --page buttons
cargo run --example gallery -- --capture shots   # macOS: PNG of every page, light and dark
```

## Build notes

- gpui 0.2.2 compiles its Metal shaders at runtime here (`runtime_shaders`, on by default), so a full Xcode install is not required.
- Tested on macOS only. The capture tool needs macOS.

## License

MIT or Apache-2.0, at your option. Lucide icons: ISC. Inter and JetBrains Mono: SIL Open Font License 1.1.
