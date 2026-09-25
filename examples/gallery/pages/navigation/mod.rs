mod paths;
mod tabs;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 7,
    slug: "navigation",
    title: "Navigation",
    summary: "Tabs, paths, pages and steps: where you are, and the ways out.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("tabs-top", 128.0, 14.0),
    Step::UpAt("tabs-top", 128.0, 14.0),
    Step::Wait(500),
    Step::Shot("tabs"),
    Step::Hover("editor-tabs"),
    Step::Wait(300),
    Step::Shot("editor-hover"),
    Step::DownAt("editor-tabs", 544.0, 16.0),
    Step::UpAt("editor-tabs", 544.0, 16.0),
    Step::Wait(300),
    Step::Shot("editor-overflow"),
    Step::Key("escape"),
    Step::DownAt("breadcrumb", 173.0, 10.0),
    Step::UpAt("breadcrumb", 173.0, 10.0),
    Step::Wait(300),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("breadcrumb-open"),
    Step::Key("enter"),
    Step::DownAt("pagination", 150.0, 10.0),
    Step::UpAt("pagination", 150.0, 10.0),
    Step::Wait(300),
    Step::Shot("pagination"),
    Step::DownAt("wizard", 531.0, 173.0),
    Step::UpAt("wizard", 531.0, 173.0),
    Step::Wait(500),
    Step::Shot("wizard"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(tabs::tabs(window, cx))
        .child(tabs::editor_tabs(window, cx))
        .child(paths::breadcrumb(window, cx))
        .child(paths::pagination(window, cx))
        .child(paths::steps(window, cx))
        .into_any_element()
}
