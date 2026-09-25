use ely_gpui_component::theme::{ActiveTheme, TextSize};
use gpui::{
    App, Div, Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div,
};

/// Titled block of demos.
pub fn section(title: impl Into<SharedString>, note: impl Into<SharedString>, cx: &App) -> Div {
    let theme = cx.theme();
    div().flex().flex_col().gap_4().pt_10().child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Md))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.colors.fg)
                    .child(title.into()),
            )
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(note.into()),
            ),
    )
}

pub fn row() -> Div {
    div().flex().flex_wrap().items_center().gap_3()
}

/// Row whose captions share one line.
pub fn specimens() -> Div {
    div().flex().flex_wrap().items_end().gap_6()
}

pub fn label(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Xs))
        .text_color(theme.colors.fg_subtle)
        .child(text.into())
}

/// Demo with a caption beneath.
pub fn specimen(caption: impl Into<SharedString>, body: impl IntoElement, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(body)
        .child(label(caption, cx))
}

/// Monospace hint naming the call.
pub fn code(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .font_family(theme.mono_family.clone())
        .text_size(theme.text_size(TextSize::Xs))
        .text_color(theme.colors.fg_muted)
        .child(text.into())
}

/// Honest note for an entry gpui cannot support.
pub fn blocked(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .gap_2()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.warning)
        .child(div().size_1p5().rounded_full().bg(theme.colors.warning))
        .child(text.into())
}

/// A demo's state, kept across frames under `key`.
pub fn keep<T: 'static>(
    key: &'static str,
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> Entity<T> {
    window.use_keyed_state(key, cx, move |_, _| init())
}

/// Replaces a demo's state and redraws.
pub fn set<T: 'static>(state: &Entity<T>, value: T, cx: &mut App) {
    state.update(cx, |state, cx| {
        *state = value;
        cx.notify();
    });
}
