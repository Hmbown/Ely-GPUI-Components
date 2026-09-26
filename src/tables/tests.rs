use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Render, SharedString, Styled, TestAppContext,
    VisualTestContext, Window, point, px,
};

use super::{Column, DataTable, Row};
use crate::theme::Theme;

/// Five selectable rows, 36 tall under a 36 tall header; it keeps what the table reports.
struct Picks(Rc<RefCell<Vec<SharedString>>>);

impl Render for Picks {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (now, store) = (self.0.borrow().clone(), self.0.clone());
        DataTable::new("picks", [Column::new("name", "Name")])
            .rows(
                (0..5)
                    .map(|ix| Row::new(format!("r{ix}"), [format!("Row {ix}").into()]))
                    .collect::<Vec<_>>(),
            )
            .selected(now)
            .on_select(move |keys, _, _| *store.borrow_mut() = keys.to_vec())
            .w(px(400.0))
    }
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn picks(seen: &Rc<RefCell<Vec<SharedString>>>) -> Vec<String> {
    seen.borrow().iter().map(|key| key.to_string()).collect()
}

#[gpui::test]
fn boxes_toggle_shift_takes_a_range_and_the_header_takes_all(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let store = seen.clone();
    let (_, cx) = cx.add_window_view(|_, _| Picks(store));
    settle(cx);
    let row = |ix: usize, x: f32| point(px(x), px(54.0 + 36.0 * ix as f32));
    cx.simulate_click(row(0, 14.0), Modifiers::none());
    settle(cx);
    assert_eq!(picks(&seen), ["r0"]);
    cx.simulate_click(
        row(3, 200.0),
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
    );
    settle(cx);
    assert_eq!(picks(&seen), ["r0", "r1", "r2", "r3"]);
    cx.simulate_click(point(px(14.0), px(18.0)), Modifiers::none());
    settle(cx);
    assert_eq!(
        picks(&seen),
        ["r0", "r1", "r2", "r3", "r4"],
        "a mixed header box takes all"
    );
    cx.simulate_click(point(px(14.0), px(18.0)), Modifiers::none());
    settle(cx);
    assert!(picks(&seen).is_empty(), "a full one clears");
}

/// Six rows in two groups, measured as they draw.
struct Grouped(Rc<std::cell::Cell<gpui::Pixels>>);

impl Render for Grouped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let tall = self.0.clone();
        let rows: Vec<Row> = (0..6)
            .map(|ix| {
                Row::new(
                    format!("r{ix}"),
                    [
                        if ix < 4 { "Europe" } else { "Asia" }.into(),
                        format!("Row {ix}").into(),
                    ],
                )
            })
            .collect();
        crate::primitives::Measure::new("measure", move |bounds, _, _| tall.set(bounds.size.height))
            .w(px(400.0))
            .child(
                DataTable::new(
                    "grouped",
                    [Column::new("region", "Region"), Column::new("name", "Name")],
                )
                .rows(rows)
                .group_by("region"),
            )
    }
}

#[gpui::test]
fn a_press_on_a_group_folds_its_rows(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let tall = Rc::new(std::cell::Cell::new(gpui::Pixels::ZERO));
    let seen = tall.clone();
    let (_, cx) = cx.add_window_view(|_, _| Grouped(seen));
    settle(cx);
    let open = tall.get();
    cx.simulate_click(point(px(60.0), px(54.0)), Modifiers::none());
    settle(cx);
    assert_eq!(
        open - tall.get(),
        px(36.0 * 4.0),
        "Europe's four rows fold away"
    );
}

/// Two rows whose names edit in place and whose detail opens; it keeps what it hears.
struct Editable(Rc<RefCell<Vec<String>>>);

impl Render for Editable {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (heard, opened) = (self.0.clone(), self.0.clone());
        DataTable::new("editable", [Column::new("name", "Name").editable()])
            .rows(vec![
                Row::new("a", ["Ada".into()]),
                Row::new("b", ["Alan".into()]),
            ])
            .detail(move |key, _, _| {
                opened.borrow_mut().push(format!("detail {key}"));
                gpui::div().h(px(40.0)).into_any_element()
            })
            .on_edit(move |row, column, text, _, _| {
                heard
                    .borrow_mut()
                    .push(format!("edit {row} {column} {text}"))
            })
            .w(px(400.0))
    }
}

#[gpui::test]
fn a_row_opens_its_detail_and_a_double_press_edits_a_cell(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        crate::forms::bind_keys(cx);
    });
    let heard = Rc::new(RefCell::new(Vec::new()));
    let seen = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Editable(seen));
    settle(cx);
    cx.simulate_click(point(px(14.0), px(54.0)), Modifiers::none());
    settle(cx);
    assert_eq!(heard.borrow().last().map(String::as_str), Some("detail a"));
    let name = point(px(120.0), px(54.0 + 36.0 + 40.0 + 24.0));
    for count in [1, 2] {
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Left,
            position: name,
            modifiers: Modifiers::none(),
            click_count: count,
            first_mouse: false,
        });
        cx.simulate_event(gpui::MouseUpEvent {
            button: gpui::MouseButton::Left,
            position: name,
            modifiers: Modifiers::none(),
            click_count: count,
        });
    }
    settle(cx);
    cx.simulate_input("Alan Kay");
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(
        heard
            .borrow()
            .iter()
            .filter(|line| line.starts_with("edit"))
            .cloned()
            .collect::<Vec<_>>(),
        ["edit b name Alan Kay"]
    );
}

/// A two-column grid of three rows that keeps every edit it hears.
struct Grid(Rc<RefCell<Vec<(usize, usize, String)>>>);

impl Render for Grid {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let heard = self.0.clone();
        let rows: Vec<Vec<SharedString>> = (0..3)
            .map(|row| vec![format!("a{row}").into(), format!("b{row}").into()])
            .collect();
        super::DataGrid::new("grid", ["A", "B"], rows)
            .on_change(move |edits, _, _| {
                heard.borrow_mut().extend(
                    edits
                        .iter()
                        .map(|(row, col, text)| (*row, *col, text.to_string())),
                )
            })
            .w(px(300.0))
            .h(px(200.0))
    }
}

#[gpui::test]
fn typing_edits_enter_keeps_and_backspace_clears_a_range(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        crate::forms::bind_keys(cx);
    });
    let heard = Rc::new(RefCell::new(Vec::new()));
    let seen = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Grid(seen));
    settle(cx);
    cx.simulate_click(point(px(48.0), px(42.0)), Modifiers::none());
    settle(cx);
    cx.simulate_keystrokes("7");
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(*heard.borrow(), [(0, 0, "7".to_string())]);
    cx.simulate_keystrokes("shift-down backspace");
    settle(cx);
    assert_eq!(
        heard.borrow()[1..],
        [(1, 0, String::new()), (2, 0, String::new())],
        "enter moved down; the range clears"
    );
}
