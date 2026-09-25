mod buttons;
mod primitives;
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
    buttons::PAGE,
    theme::PAGE,
];

pub fn find(slug: &str) -> Option<usize> {
    ALL.iter().position(|page| page.slug == slug)
}
