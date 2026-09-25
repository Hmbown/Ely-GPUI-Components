mod long;
mod rows;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 13,
    slug: "lists",
    title: "Lists & Trees",
    summary: "Rows to read, pick, reorder and swipe; lists long enough to need care.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("pick", 20.0, 52.0),
    Step::UpAt("pick", 20.0, 52.0),
    Step::Key("shift-down"),
    Step::Wait(200),
    Step::Shot("selected-range"),
    Step::Swipe("swipe-row", -120.0),
    Step::Wait(500),
    Step::Shot("swiped-open"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(rows::items(cx))
        .child(rows::selectable(window, cx))
        .child(rows::sortable(window, cx))
        .child(rows::swipeable(window, cx))
        .child(long::virtual_list(cx))
        .child(long::infinite(window, cx))
        .child(long::grouped(cx))
        .into_any_element()
}
