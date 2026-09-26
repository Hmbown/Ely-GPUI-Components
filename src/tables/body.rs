use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, Rems,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::*,
};

use super::{Align, Column, Row, cell::draw, model::tint};
use crate::{
    forms::{CheckState, check_mark},
    lists::Pick,
    theme::{ActiveTheme, TextSize},
};

pub(super) type Select = Rc<dyn Fn(usize, Pick, &mut gpui::Window, &mut App)>;

/// Gives a cell its column's width: fixed, or a share of what is left.
pub(super) fn sized<E: Styled>(cell: E, column: &Column) -> E {
    let cell = match column.width {
        Some(width) => cell.w(width).flex_none(),
        None => cell.flex_1().min_w_0(),
    };
    if column.align == Align::End {
        cell.justify_end()
    } else {
        cell
    }
}

/// What drawing a row needs, shared by pages and by a long table's list.
pub(super) struct Body {
    pub id: ElementId,
    pub columns: Rc<Vec<Column>>,
    pub rows: Rc<Vec<Row>>,
    pub keys: Rc<Vec<SharedString>>,
    pub shown: Rc<Vec<usize>>,
    pub offset: usize,
    pub selected: Option<Rc<Vec<SharedString>>>,
    pub select: Option<Select>,
    pub ranges: Rc<Vec<Option<(f64, f64)>>>,
    pub height: Rems,
    pub check: Rems,
}

impl Body {
    pub(super) fn row(&self, at: usize, cx: &App) -> AnyElement {
        let ix = self.shown[at];
        let key = self.keys[ix].clone();
        let on = self
            .selected
            .as_ref()
            .is_some_and(|selected| selected.contains(&key));
        let theme = cx.theme();
        let colors = &theme.colors;
        let position = self.offset + at;
        let press = self.select.clone();
        div()
            .id((self.id.clone(), format!("row-{key}")))
            .flex()
            .items_center()
            .w_full()
            .h(self.height)
            .border_b_1()
            .border_color(colors.border)
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(colors.fg)
            .when(on, |row| row.bg(colors.active))
            .when(!on, |row| row.hover(|style| style.bg(colors.hover)))
            .when_some(self.select.clone(), |row, select| {
                row.cursor_pointer()
                    .child(
                        div()
                            .flex_none()
                            .w(self.check)
                            .flex()
                            .justify_center()
                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                cx.stop_propagation();
                                select(position, Pick::Toggle, window, cx);
                            })
                            .child(check_mark(
                                if on { CheckState::On } else { CheckState::Off },
                                false,
                                false,
                                0,
                                cx,
                            )),
                    )
                    .on_click(move |event, window, cx| {
                        let held = event.modifiers();
                        let how = match (held.shift, held.platform) {
                            (true, _) => Pick::Range,
                            (false, true) => Pick::Toggle,
                            _ => Pick::One,
                        };
                        if let Some(press) = &press {
                            press(position, how, window, cx);
                        }
                    })
            })
            .children(self.columns.iter().enumerate().map(|(col, column)| {
                let cell = &self.rows[ix].cells[col];
                let tinted = self.ranges[col]
                    .zip(cell.number())
                    .map(|(range, value)| tint(value, range, colors));
                sized(div().h_full().flex().items_center().px_3(), column)
                    .when_some(tinted, |cell, bg| cell.bg(bg))
                    .child(draw(
                        cell,
                        column,
                        (self.id.clone(), format!("cell-{key}-{col}")).into(),
                        cx,
                    ))
            }))
            .into_any_element()
    }
}
