mod buttons;
mod primitives;
mod theme;

use gpui::{AnyElement, App, Window};

pub struct Page {
    pub number: u8,
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub render: fn(&mut Window, &mut App) -> AnyElement,
}

pub const ALL: &[Page] = &[primitives::PAGE, buttons::PAGE, theme::PAGE];

pub fn find(slug: &str) -> Option<usize> {
    ALL.iter().position(|page| page.slug == slug)
}
