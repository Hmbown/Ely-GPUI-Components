mod formats;
mod numbers;
mod rich;
mod text;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 6,
    slug: "forms",
    title: "Forms",
    summary: "Text, choices, dates, colors, files: every way to say what you mean.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("input-name"),
    Step::Key("secondary-a"),
    Step::Type("Ada Lovelace"),
    Step::Key("shift-left"),
    Step::Key("shift-left"),
    Step::Key("shift-left"),
    Step::Wait(300),
    Step::Shot("typed"),
    Step::Click("textarea"),
    Step::Key("secondary-a"),
    Step::Type("First line of notes"),
    Step::Key("enter"),
    Step::Type("Second line, longer, so the words wrap when they reach the edge of the box"),
    Step::Wait(300),
    Step::Shot("textarea"),
    Step::Click("password"),
    Step::Key("secondary-a"),
    Step::Type("hunter2"),
    Step::Wait(200),
    Step::Shot("password"),
    Step::Click("search"),
    Step::Wait(300),
    Step::Shot("search-history"),
    Step::Click("number"),
    Step::Key("up"),
    Step::Key("up"),
    Step::Wait(300),
    Step::Shot("number"),
    Step::DownAt("scrub", 12.0, 14.0),
    Step::DragTo("scrub", 72.0, 14.0),
    Step::UpAt("scrub", 72.0, 14.0),
    Step::Wait(300),
    Step::Shot("scrubbed"),
    Step::Click("email"),
    Step::Key("secondary-a"),
    Step::Type("ada@example"),
    Step::Click("phone"),
    Step::Key("secondary-a"),
    Step::Type("4155550132"),
    Step::Click("masked"),
    Step::Key("secondary-a"),
    Step::Type("ab1234"),
    Step::Wait(300),
    Step::Shot("formats"),
    Step::Click("pin"),
    Step::Key("secondary-a"),
    Step::Type("4821"),
    Step::Wait(300),
    Step::Shot("pin"),
    Step::Click("inline"),
    Step::Key("secondary-a"),
    Step::Type("Q3 planning"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("inline"),
    Step::Click("mention"),
    Step::Key("secondary-a"),
    Step::Type("Ask @gr"),
    Step::Wait(300),
    Step::Shot("mention"),
    Step::Key("enter"),
    Step::Type("about #rel"),
    Step::Key("down"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("mentioned"),
    Step::Click("tags"),
    Step::Type("motion,design"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("tags"),
    Step::Click("regex"),
    Step::Key("secondary-a"),
    Step::Type("^(ab+"),
    Step::Wait(300),
    Step::Shot("regex"),
    Step::Click("expression"),
    Step::Key("secondary-a"),
    Step::Type("price * qty + max(2, 4)"),
    Step::Wait(300),
    Step::Shot("expression"),
    Step::Click("hotkey"),
    Step::Key("cmd-shift-k"),
    Step::Wait(300),
    Step::Shot("hotkey"),
    Step::DownAt("path", 380.0, 16.0),
    Step::UpAt("path", 380.0, 16.0),
    Step::Wait(1200),
    Step::CancelPanel,
    Step::Wait(600),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(text::input(window, cx))
        .child(text::text_area(window, cx))
        .child(text::password(window, cx))
        .child(text::search(window, cx))
        .child(numbers::number_input(window, cx))
        .child(numbers::scrub(window, cx))
        .child(numbers::money(window, cx))
        .child(formats::email_url(window, cx))
        .child(formats::phone_masked(window, cx))
        .child(formats::pin(window, cx))
        .child(formats::inline_edit(window, cx))
        .child(text::group(window, cx))
        .child(rich::mention(window, cx))
        .child(rich::tags(window, cx))
        .child(rich::rows(cx))
        .child(rich::path(window, cx))
        .child(rich::regex(window, cx))
        .child(rich::expression(window, cx))
        .child(rich::hotkey(window, cx))
        .into_any_element()
}
