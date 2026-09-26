mod core;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 14,
    slug: "tables",
    title: "Tables",
    summary: "Rows and columns that sort, filter, page and select, and stay quick at a hundred thousand rows.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("orders", 440.0, 16.0),
    Step::UpAt("orders", 440.0, 16.0),
    Step::Wait(200),
    Step::Shot("sorted"),
    Step::DownAt("orders", 18.0, 54.0),
    Step::UpAt("orders", 18.0, 54.0),
    Step::DownAt("orders", 18.0, 126.0),
    Step::UpAt("orders", 18.0, 126.0),
    Step::Wait(200),
    Step::Shot("selected"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(core::plain(cx))
        .child(core::orders(window, cx))
        .child(core::long(window, cx))
        .child(core::heatmap(cx))
        .into_any_element()
}
