use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, IntoElement, Modifiers, Render, SharedString, Styled, TestAppContext,
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
