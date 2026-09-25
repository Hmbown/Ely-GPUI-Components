mod loading;
mod progress;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 11,
    slug: "motion",
    title: "Loading & Motion",
    summary: "Waiting, shown honestly, and the motion everything else moves with.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("save"),
    Step::Wait(250),
    Step::Shot("loading-button"),
    Step::Click("load-more"),
    Step::Wait(300),
    Step::Shot("load-more-busy"),
    Step::Wait(1200),
    Step::Shot("load-more-done"),
    Step::Click("shimmer-load"),
    Step::Wait(500),
    Step::Shot("shimmer-loaded"),
    Step::Click("shimmer-load"),
    Step::Click("progress-step"),
    Step::Wait(400),
    Step::Shot("progress"),
    Step::Click("overlay-reload"),
    Step::Wait(300),
    Step::Shot("overlay"),
    Step::Wait(1500),
    Step::Click("refresh-run"),
    Step::Wait(300),
    Step::Shot("refreshing"),
    Step::Click("uploads-tick"),
    Step::Wait(400),
    Step::Shot("uploads"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(loading::spinners(cx))
        .child(loading::skeletons(cx))
        .child(loading::shimmer(window, cx))
        .child(loading::buttons(window, cx))
        .child(loading::more(window, cx))
        .child(progress::bars(window, cx))
        .child(progress::overlay(window, cx))
        .child(progress::lazy(cx))
        .child(progress::suspense(window, cx))
        .child(progress::refresh(window, cx))
        .child(progress::uploads(window, cx))
        .into_any_element()
}
