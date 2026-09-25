use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    Context, Entity, IntoElement, Modifiers, ParentElement, Pixels, Point, Render, ScrollDelta,
    ScrollWheelEvent, SharedString, Styled, TestAppContext, TouchPhase, VisualTestContext, Window,
    div, point, px,
};

use super::{
    InfiniteList, ListItem, SelectableList, SortableList, SwipeAction, SwipeableListItem,
    VirtualList,
};
use crate::{
    primitives::{IconName, Measure},
    theme::Theme,
};

fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
}

/// Frames 2ms apart, past reduced motion's 1ms glides.
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

const NAMES: [&str; 4] = ["a", "b", "c", "d"];

/// A multiple-choice list of four rows that keeps what it reports.
struct Picks(Vec<SharedString>);

impl Render for Picks {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        NAMES
            .iter()
            .fold(SelectableList::new("picks").multiple(), |list, name| {
                list.row(*name, ListItem::new(*name, *name))
            })
            .selected(self.0.clone())
            .on_change(move |keys, _, cx| view.update(cx, |picks, _| picks.0 = keys.to_vec()))
            .w(px(240.0))
            .h(px(200.0))
    }
}

fn picks(view: &Entity<Picks>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |picks, _| {
        picks.0.iter().map(|key| key.to_string()).collect()
    })
}

/// Row `ix`'s left padding: rows are 32 tall with 2 between.
fn row(ix: usize) -> Point<Pixels> {
    point(px(6.0), px(16.0 + 34.0 * ix as f32))
}

#[gpui::test]
fn presses_pick_one_toggle_with_cmd_and_range_with_shift(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Picks(Vec::new()));
    settle(cx);
    cx.simulate_click(row(0), Modifiers::none());
    settle(cx);
    assert_eq!(picks(&view, cx), ["a"]);
    cx.simulate_click(
        row(2),
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
    );
    settle(cx);
    assert_eq!(picks(&view, cx), ["a", "b", "c"]);
    cx.simulate_click(
        row(1),
        Modifiers {
            platform: true,
            ..Modifiers::none()
        },
    );
    settle(cx);
    assert_eq!(picks(&view, cx), ["a", "c"]);
    cx.simulate_keystrokes("down shift-down");
    settle(cx);
    assert_eq!(
        picks(&view, cx),
        ["c", "d"],
        "down picks one, shift-down extends from it"
    );
    cx.simulate_keystrokes("cmd-a");
    settle(cx);
    assert_eq!(picks(&view, cx), NAMES);
}

/// A row with two actions, 144 across; it reports where the row sits and which action ran.
struct Swiped {
    left: Rc<Cell<Pixels>>,
    ran: Rc<RefCell<Vec<&'static str>>>,
}

impl Render for Swiped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (left, archive, delete) = (self.left.clone(), self.ran.clone(), self.ran.clone());
        div().w(px(320.0)).child(
            SwipeableListItem::new(
                "swiped",
                Measure::new("row", move |bounds, _, _| left.set(bounds.origin.x))
                    .h(px(40.0))
                    .child("Invoice 2419"),
            )
            .action(SwipeAction::new(
                "Archive",
                IconName::Archive,
                move |_, _| archive.borrow_mut().push("archive"),
            ))
            .action(SwipeAction::new("Delete", IconName::Trash2, move |_, _| {
                delete.borrow_mut().push("delete")
            })),
        )
    }
}

fn scroll(cx: &mut VisualTestContext, phase: TouchPhase, x: f32, y: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(160.0), px(20.0)),
        delta: ScrollDelta::Pixels(point(px(x), px(y))),
        modifiers: Modifiers::none(),
        touch_phase: phase,
    });
    settle(cx);
}

#[gpui::test]
fn a_sideways_swipe_opens_the_row_and_an_action_shuts_it(cx: &mut TestAppContext) {
    setup(cx);
    let (left, ran) = (
        Rc::new(Cell::new(Pixels::ZERO)),
        Rc::new(RefCell::new(Vec::new())),
    );
    let (seen, done) = (left.clone(), ran.clone());
    let (_, cx) = cx.add_window_view(|_, _| Swiped {
        left: seen,
        ran: done,
    });
    settle(cx);
    scroll(cx, TouchPhase::Started, 0.0, 0.0);
    scroll(cx, TouchPhase::Moved, -2.0, -30.0);
    scroll(cx, TouchPhase::Moved, -100.0, 0.0);
    scroll(cx, TouchPhase::Ended, 0.0, 0.0);
    assert_eq!(
        left.get(),
        Pixels::ZERO,
        "a gesture that starts upright is the list's"
    );
    scroll(cx, TouchPhase::Started, 0.0, 0.0);
    scroll(cx, TouchPhase::Moved, -50.0, 4.0);
    assert_eq!(left.get(), px(-50.0), "the row follows the fingers");
    scroll(cx, TouchPhase::Moved, -40.0, 0.0);
    scroll(cx, TouchPhase::Ended, 0.0, 0.0);
    assert_eq!(left.get(), px(-144.0), "past half its reach, it snaps open");
    cx.simulate_click(point(px(300.0), px(20.0)), Modifiers::none());
    settle(cx);
    assert_eq!(*ran.borrow(), ["delete"]);
    assert_eq!(left.get(), Pixels::ZERO, "an action shuts the row");
}

/// Rows in a short scrolling box that count how often they asked for more.
struct Paging {
    rows: usize,
    asked: usize,
}

impl Render for Paging {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div().h(px(400.0)).child(
            InfiniteList::new("paging")
                .children((0..self.rows).map(|ix| div().h(px(20.0)).child(format!("Row {ix}"))))
                .on_more(move |_, cx| view.update(cx, |paging, _| paging.asked += 1)),
        )
    }
}

#[gpui::test]
fn the_end_in_view_asks_for_more_once_a_page(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Paging { rows: 3, asked: 0 });
    settle(cx);
    assert_eq!(view.read_with(cx, |paging, _| paging.asked), 1);
    settle(cx);
    assert_eq!(
        view.read_with(cx, |paging, _| paging.asked),
        1,
        "staying in view asks no more"
    );
    view.update(cx, |paging, cx| {
        paging.rows = 6;
        cx.notify();
    });
    settle(cx);
    assert_eq!(
        view.read_with(cx, |paging, _| paging.asked),
        2,
        "a new page asks again"
    );
}

/// Three sortable rows; the moves the owner hears.
struct Sorting(Rc<RefCell<Vec<(usize, usize)>>>);

impl Render for Sorting {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let moves = self.0.clone();
        NAMES[..3]
            .iter()
            .fold(SortableList::new("sorting"), |list, name| {
                list.row(*name, ListItem::new(*name, *name))
            })
            .on_reorder(move |from, to, _, _| moves.borrow_mut().push((from, to)))
    }
}

#[gpui::test]
fn alt_with_arrows_moves_the_current_row(cx: &mut TestAppContext) {
    setup(cx);
    let moves = Rc::new(RefCell::new(Vec::new()));
    let heard = moves.clone();
    let (_, cx) = cx.add_window_view(|_, _| Sorting(heard));
    settle(cx);
    cx.simulate_click(row(0), Modifiers::none());
    settle(cx);
    cx.simulate_keystrokes("alt-down down alt-up alt-up");
    settle(cx);
    assert_eq!(*moves.borrow(), [(0, 1), (2, 1), (1, 0)]);
}

/// A thousand rows, 20 tall, in a box 200 tall; it keeps the rows it was asked to build.
struct Long(Rc<RefCell<Vec<usize>>>);

impl Render for Long {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let built = self.0.clone();
        VirtualList::new("long", 1000, move |ix, _, _| {
            built.borrow_mut().push(ix);
            div()
                .h(px(20.0))
                .child(format!("Row {ix}"))
                .into_any_element()
        })
        .h(px(200.0))
    }
}

#[gpui::test]
fn a_long_list_builds_only_what_is_near_the_view(cx: &mut TestAppContext) {
    setup(cx);
    let built = Rc::new(RefCell::new(Vec::new()));
    let seen = built.clone();
    let (_, cx) = cx.add_window_view(|_, _| Long(seen));
    settle(cx);
    let most = built.borrow().iter().copied().max().expect("some rows");
    assert!((9..60).contains(&most), "built up to row {most} of 1000");
}
