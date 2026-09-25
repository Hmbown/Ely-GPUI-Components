mod people;
mod records;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 12,
    slug: "data",
    title: "Data Display",
    summary: "People, numbers and records, shown plainly.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("badge-more"),
    Step::Wait(200),
    Step::Shot("count-rolling"),
    Step::Hover("avatar-upload"),
    Step::Wait(300),
    Step::Shot("avatar-hover"),
    Step::Click("stats-next"),
    Step::Wait(300),
    Step::Shot("stats-next"),
    Step::Click("feed-post"),
    Step::Wait(300),
    Step::Shot("feed-arrival"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(people::badges(window, cx))
        .child(people::tags(window, cx))
        .child(people::avatars(window, cx))
        .child(people::groups(cx))
        .child(people::numbers(window, cx))
        .child(records::descriptions(cx))
        .child(records::properties(window, cx))
        .child(records::timelines(cx))
        .child(records::feed(window, cx))
        .child(records::changelog(cx))
        .into_any_element()
}
