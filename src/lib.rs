mod assets;

pub mod buttons;
pub mod motion;
pub mod primitives;
pub mod theme;

pub use assets::Assets;

use gpui::{App, KeyBinding};

use crate::primitives::{FocusNext, FocusPrev, IconName};

/// Loads fonts and the theme, binds Tab. Call once, first.
pub fn init(cx: &mut App) {
    match cx.asset_source().load(IconName::Check.path()) {
        Ok(Some(_)) => {}
        Ok(None) => panic!("ely: pass `ely_gpui_component::Assets` to `Application::with_assets`"),
        Err(error) => panic!("ely: asset source failed: {error:#}"),
    }
    assets::load_fonts(cx).expect("ely: embedded fonts failed to register");
    theme::Theme::init(cx);
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrev, None),
    ]);
}
