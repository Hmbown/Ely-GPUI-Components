use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, Entity, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Pixels, RenderOnce, ScrollHandle, SharedString, Styled, Window, canvas, div,
    prelude::*,
};

use super::{
    Choice,
    check::tab_stop,
    input::text_size,
    options::{OnValue, Pick, Popup, step},
};
use crate::{
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, Radius},
};

/// An open list's place and cursor, kept between frames.
#[derive(Default)]
pub(crate) struct Picker {
    pub open: bool,
    pub highlighted: usize,
    pub anchor: Bounds<Pixels>,
    pub scroll: ScrollHandle,
}

impl Picker {
    pub fn show(state: &Entity<Picker>, open: bool, highlighted: usize, cx: &mut App) {
        state.update(cx, |picker, cx| {
            picker.open = open;
            picker.highlighted = highlighted;
            picker.scroll.scroll_to_item(highlighted);
            cx.notify();
        });
    }
}

/// Sizes an invisible layer to its parent and records the bounds in `state`.
pub(crate) fn measure_anchor(state: Entity<Picker>) -> impl IntoElement {
    canvas(
        move |bounds, _, cx| {
            if state.read(cx).anchor != bounds {
                state.update(cx, |picker, _| picker.anchor = bounds);
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Up, Down, Home and End over `rows` from `at`; `None` for other keys.
pub(crate) fn moved(event: &KeyDownEvent, rows: &[Choice], at: usize) -> Option<usize> {
    let last = rows.len() - 1;
    match event.keystroke.key.as_str() {
        "down" => Some(step(rows, at, 1)),
        "up" => Some(step(rows, at, -1)),
        "home" => Some(step(rows, last, 1)),
        "end" => Some(step(rows, 0, -1)),
        _ => None,
    }
}

/// A button that opens a list and shows the one chosen.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    choices: Vec<Choice>,
    selected: Option<SharedString>,
    placeholder: SharedString,
    size: ControlSize,
    disabled: bool,
    on_change: Option<OnValue>,
}

impl Select {
    pub fn new(id: impl Into<ElementId>, choices: impl IntoIterator<Item = Choice>) -> Self {
        Self {
            id: id.into(),
            choices: choices.into_iter().collect(),
            selected: None,
            placeholder: SharedString::from("Choose…"),
            size: ControlSize::default(),
            disabled: false,
            on_change: None,
        }
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(
            !self.choices.is_empty(),
            "select {:?} has no choices",
            self.id
        );
        let focus = tab_stop(
            (self.id.clone(), "focus").into(),
            !self.disabled,
            window,
            cx,
        );
        let focused = focus.is_focused(window);
        let picker =
            window.use_keyed_state((self.id.clone(), "picker"), cx, |_, _| Picker::default());
        if picker.read(cx).open && !focused {
            log::info!("select {:?}: closed on blur", self.id);
            picker.update(cx, |picker, _| picker.open = false);
        }
        let choices = Rc::new(self.choices);
        let current = self
            .selected
            .as_ref()
            .and_then(|value| choices.iter().position(|choice| choice.value == *value));
        let start = current.unwrap_or_else(|| step(&choices, choices.len() - 1, 1));
        if picker.read(cx).highlighted >= choices.len() {
            picker.update(cx, |picker, _| picker.highlighted = start);
        }
        let (open, highlighted, anchor, scroll) = {
            let picker = picker.read(cx);
            (
                picker.open,
                picker.highlighted,
                picker.anchor,
                picker.scroll.clone(),
            )
        };
        let pick: Pick = {
            let (id, choices, picker, on_change) = (
                self.id.clone(),
                choices.clone(),
                picker.clone(),
                self.on_change,
            );
            Rc::new(move |ix, window, cx| {
                if choices[ix].disabled {
                    return;
                }
                let value = choices[ix].value.clone();
                log::info!("select {id:?}: {value}");
                Picker::show(&picker, false, ix, cx);
                if let Some(on_change) = &on_change {
                    on_change(&value, window, cx);
                }
            })
        };
        let theme = cx.theme();
        let colors = &theme.colors;
        let shown = current.map(|ix| &choices[ix]);
        let (toggle, keys, rows, close) = (
            picker.clone(),
            picker.clone(),
            choices.clone(),
            picker.clone(),
        );
        let enter = pick.clone();
        let chosen: Vec<SharedString> = shown.iter().map(|choice| choice.value.clone()).collect();
        div()
            .id(self.id.clone())
            .track_focus(&focus)
            .relative()
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .h(theme.control_height(self.size))
            .px(theme.control_padding(self.size))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(if focused {
                colors.focus
            } else {
                colors.border_strong
            })
            .bg(if self.disabled {
                colors.sunken
            } else {
                colors.surface
            })
            .text_size(theme.text_size(text_size(self.size)))
            .when(!self.disabled, |trigger| {
                trigger
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        Picker::show(&toggle, !open, start, cx)
                    })
                    .on_key_down(move |event, window, cx| {
                        let at = keys.read(cx).highlighted;
                        let key = event.keystroke.key.as_str();
                        if !open {
                            if matches!(key, "down" | "up" | "enter" | "space") {
                                cx.stop_propagation();
                                Picker::show(&keys, true, start, cx);
                            }
                            return;
                        }
                        if let Some(to) = moved(event, &rows, at) {
                            cx.stop_propagation();
                            Picker::show(&keys, true, to, cx);
                        } else if matches!(key, "enter" | "space") {
                            cx.stop_propagation();
                            enter(at, window, cx);
                        } else if key == "escape" {
                            cx.stop_propagation();
                            Picker::show(&keys, false, at, cx);
                        }
                    })
            })
            .when_some(shown.and_then(|choice| choice.icon), |trigger, icon| {
                trigger.child(Icon::new(icon).size(IconSize::Sm).color(colors.fg_muted))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_color(match (shown, self.disabled) {
                        (_, true) => colors.fg_disabled,
                        (Some(_), false) => colors.fg,
                        (None, false) => colors.fg_subtle,
                    })
                    .child(shown.map_or(self.placeholder, |choice| choice.label.clone())),
            )
            .child(
                Icon::new(IconName::ChevronsUpDown)
                    .size(IconSize::Xs)
                    .color(colors.fg_subtle),
            )
            .child(measure_anchor(picker))
            .when(open, |trigger| {
                trigger.child(
                    Popup {
                        id: (self.id, "list").into(),
                        anchor,
                        rows: &choices,
                        highlighted: Some(highlighted),
                        checked: Some(&chosen),
                        pick,
                        dismiss: Some(Rc::new(move |_, cx| {
                            let at = close.read(cx).highlighted;
                            Picker::show(&close, false, at, cx)
                        })),
                        scroll: Some(&scroll),
                    }
                    .render(window, cx),
                )
            })
    }
}
