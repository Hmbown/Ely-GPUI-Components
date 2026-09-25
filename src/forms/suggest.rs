use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, SharedString, Styled, anchored, deferred, div, point, prelude::*,
};

use crate::theme::{ActiveTheme, Elevation, Radius, TextSize};

/// One row in a suggestion list: a label and a quiet note.
#[derive(Clone, Debug)]
pub(crate) struct Suggestion {
    pub label: SharedString,
    pub note: Option<SharedString>,
}

type Pick = Rc<dyn Fn(usize, &mut gpui::Window, &mut App)>;
pub(crate) type Dismiss = Rc<dyn Fn(&mut gpui::Window, &mut App)>;

/// A list floating under `anchor`. Rows keep focus where it was.
pub(crate) fn suggestion_list(
    id: impl Into<ElementId>,
    anchor: Bounds<Pixels>,
    rows: &[Suggestion],
    highlighted: usize,
    pick: Pick,
    dismiss: Option<Dismiss>,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme();
    let colors = &theme.colors;
    let list = div()
        .id(id)
        .min_w(theme.tooltip_max_width())
        .p_1()
        .flex()
        .flex_col()
        .rounded(theme.radius(Radius::Lg))
        .bg(colors.overlay)
        .border_1()
        .border_color(colors.border)
        .shadow(theme.elevation(Elevation::Floating))
        .text_size(theme.text_size(TextSize::Sm))
        .when_some(dismiss, |list, dismiss| {
            list.on_mouse_down_out(move |_, window, cx| dismiss(window, cx))
        })
        .children(rows.iter().enumerate().map(|(ix, row)| {
            let pick = pick.clone();
            div()
                .id(("suggestion", ix))
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .px_2()
                .py_1()
                .rounded(theme.radius(Radius::Md))
                .text_color(colors.fg)
                .when(ix == highlighted, |row| row.bg(colors.hover))
                .hover(|style| style.bg(colors.hover))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    pick(ix, window, cx);
                })
                .child(row.label.clone())
                .when_some(row.note.clone(), |row, note| {
                    row.child(div().text_color(colors.fg_subtle).child(note))
                })
        }));
    deferred(
        anchored()
            .position(point(anchor.left(), anchor.bottom()))
            .child(div().mt_1().child(list)),
    )
    .with_priority(1)
    .into_any_element()
}
