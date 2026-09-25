mod assets;

pub mod buttons;
pub mod motion;
pub mod primitives;
pub mod theme;

pub use assets::Assets;

use gpui::App;

use crate::primitives::IconName;

/// Loads fonts and the theme. Call once, first.
pub fn init(cx: &mut App) {
    let probe = IconName::Check.path();
    let wired = cx.asset_source().load(probe).ok().flatten().is_some();
    assert!(
        wired,
        "ely: pass `ely_gpui_component::Assets` to `Application::with_assets`"
    );
    assets::load_fonts(cx).expect("ely: embedded fonts failed to register");
    theme::Theme::init(cx);
}
