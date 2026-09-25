use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, Corner, Div, ElementId, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, ScrollHandle, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, anchored, deferred, div, prelude::*,
};

use crate::{
    motion,
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, Elevation, IconSize, Radius, TextSize},
};

/// One option: a value, the label shown for it, and an optional icon and note.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub value: SharedString,
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub note: Option<SharedString>,
    pub disabled: bool,
}

impl Choice {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            note: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Quiet text at the row's end, such as a count or a shortcut.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

pub(crate) type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;
pub(crate) type Run = Rc<dyn Fn(&mut Window, &mut App)>;
pub(crate) type OnFlag = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub(crate) type OnNumber = Rc<dyn Fn(f64, &mut Window, &mut App)>;
pub(crate) type OnValue = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
pub(crate) type OnValues = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;

/// The next row from `from` that is not disabled, stepping by `by` and wrapping.
pub(crate) fn step(rows: &[Choice], from: usize, by: isize) -> usize {
    let count = rows.len() as isize;
    let mut at = from as isize;
    for _ in 0..count {
        at = (at + by).rem_euclid(count);
        if !rows[at as usize].disabled {
            return at as usize;
        }
    }
    from
}

/// One row: a check column when `checked` is known, then icon, label and note.
pub(crate) fn option_row(
    id: impl Into<ElementId>,
    choice: &Choice,
    highlighted: bool,
    checked: Option<bool>,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let fg = if choice.disabled {
        colors.fg_disabled
    } else {
        colors.fg
    };
    div()
        .id(id)
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded(theme.radius(Radius::Md))
        .text_color(fg)
        .when(highlighted && !choice.disabled, |row| row.bg(colors.hover))
        .when(!choice.disabled, |row| {
            row.cursor_pointer().hover(|style| style.bg(colors.hover))
        })
        .when_some(checked, |row, on| {
            row.child(
                div()
                    .flex_none()
                    .size(theme.icon_size(IconSize::Sm))
                    .when(on, |mark| {
                        mark.child(Icon::new(IconName::Check).size(IconSize::Sm).color(fg))
                    }),
            )
        })
        .when_some(choice.icon, |row, icon| {
            row.child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_muted))
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(choice.label.clone()),
        )
        .when_some(choice.note.clone(), |row, note| {
            row.child(div().text_color(colors.fg_subtle).child(note))
        })
}

/// The card a floating list sits on.
pub(crate) fn surface(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    div()
        .id(id)
        .rounded(theme.radius(Radius::Lg))
        .bg(colors.overlay)
        .border_1()
        .border_color(colors.border)
        .shadow(theme.elevation(Elevation::Floating))
        .text_size(theme.text_size(TextSize::Sm))
}

/// Floats `content` of about `rows` rows under `anchor`, or over it when only above has room.
pub(crate) fn float(
    anchor: Bounds<Pixels>,
    rows: usize,
    content: impl IntoElement,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme();
    let rem = window.rem_size();
    let row = theme.control_height(ControlSize::Md).to_pixels(rem);
    let height = (row * (rows + 1) as f32).min(theme.list_max_height().to_pixels(rem));
    let below = window.viewport_size().height - anchor.bottom();
    let up = height > below && anchor.top() > below;
    let enter =
        Animation::new(motion::duration(motion::FAST, cx)).with_easing(motion::ease_out_cubic);
    let placed = if up {
        anchored()
            .position(anchor.origin)
            .anchor(Corner::BottomLeft)
            .child(
                div()
                    .child(content)
                    .with_animation("popup-in", enter, |popup, t| {
                        popup.opacity(t).mb(motion::NUDGE * (2.0 - t))
                    }),
            )
    } else {
        anchored()
            .position(anchor.bottom_left())
            .child(
                div()
                    .child(content)
                    .with_animation("popup-in", enter, |popup, t| {
                        popup.opacity(t).mt(motion::NUDGE * (2.0 - t))
                    }),
            )
    };
    deferred(placed.snap_to_window())
        .with_priority(1)
        .into_any_element()
}

/// A list floating under `anchor`, at least as wide. Rows leave focus where it was.
pub(crate) struct Popup<'a> {
    pub id: ElementId,
    pub anchor: Bounds<Pixels>,
    pub rows: &'a [Choice],
    pub highlighted: Option<usize>,
    pub checked: Option<&'a [SharedString]>,
    pub pick: Pick,
    pub dismiss: Option<Run>,
    pub scroll: Option<&'a ScrollHandle>,
}

impl Popup<'_> {
    pub fn render(self, window: &Window, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let (anchor, pick) = (self.anchor, self.pick);
        let rows = self.rows.iter().enumerate().map(|(ix, choice)| {
            let pick = pick.clone();
            let checked = self.checked.map(|values| values.contains(&choice.value));
            option_row(
                ("option", ix),
                choice,
                self.highlighted == Some(ix),
                checked,
                cx,
            )
            .when(!choice.disabled, |row| {
                row.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    pick(ix, window, cx);
                })
            })
        });
        let list = surface(self.id, cx)
            .min_w(theme.tooltip_max_width())
            .when(anchor.size.width > Pixels::ZERO, |list| {
                list.min_w(anchor.size.width)
            })
            .max_h(theme.list_max_height())
            .overflow_y_scroll()
            .when_some(self.scroll, |list, handle| list.track_scroll(handle))
            .p_1()
            .flex()
            .flex_col()
            .when_some(self.dismiss, |list, dismiss| {
                list.on_mouse_down_out(move |_, window, cx| dismiss(window, cx))
            })
            .children(rows);
        float(anchor, self.rows.len(), list, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{Choice, step};

    #[test]
    fn stepping_wraps_and_skips_disabled_rows() {
        let rows = [
            Choice::new("a", "A"),
            Choice::new("b", "B").disabled(),
            Choice::new("c", "C"),
        ];
        assert_eq!(step(&rows, 0, 1), 2);
        assert_eq!(step(&rows, 2, 1), 0);
        assert_eq!(step(&rows, 0, -1), 2);
        let none = [Choice::new("a", "A").disabled()];
        assert_eq!(step(&none, 0, 1), 0);
    }
}
