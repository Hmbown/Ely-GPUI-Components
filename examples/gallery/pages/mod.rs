mod buttons;
mod forms;
mod layout;
mod menus;
mod navigation;
mod overlays;
mod primitives;
mod shell;
mod theme;
mod typography;

use gpui::{AnyElement, App, Window};

use crate::script::Step;

pub struct Page {
    pub number: u8,
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub render: fn(&mut Window, &mut App) -> AnyElement,
    /// Interactions shot after the static pages.
    pub script: &'static [Step],
}

pub const ALL: &[Page] = &[
    primitives::PAGE,
    typography::PAGE,
    layout::PAGE,
    shell::PAGE,
    buttons::PAGE,
    forms::PAGE,
    navigation::PAGE,
    menus::PAGE,
    overlays::PAGE,
    theme::PAGE,
];

pub use shell::{open_about, open_managed};

pub fn find(slug: &str) -> Option<usize> {
    ALL.iter().position(|page| page.slug == slug)
}
