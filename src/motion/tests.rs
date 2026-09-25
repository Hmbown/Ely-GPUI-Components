use std::{cell::Cell, rc::Rc};

use gpui::{
    Context, InteractiveElement, IntoElement, Modifiers, ParentElement, Render, ScrollDelta,
    ScrollWheelEvent, StatefulInteractiveElement, Styled, TestAppContext, TouchPhase,
    VisualTestContext, Window, div, point, px,
};

use super::{LazyLoad, LoadingOverlay, ProgressBar};
use crate::theme::Theme;

/// A pressable box under a loading veil, and a lazy row far down a scroll.
struct Bench {
    loading: bool,
    presses: usize,
    built: Rc<Cell<usize>>,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (view, built) = (cx.entity(), self.built.clone());
        div()
            .child(
                LoadingOverlay::new("veil", self.loading).child(
                    div()
                        .id("under")
                        .w(px(200.0))
                        .h(px(100.0))
                        .on_click(move |_, _, cx| view.update(cx, |bench, _| bench.presses += 1)),
                ),
            )
            .child(
                div()
                    .id("scroll")
                    .h(px(100.0))
                    .overflow_y_scroll()
                    .child(div().h(px(1000.0)))
                    .child(LazyLoad::new("row", px(20.0), move |_, _| {
                        built.set(built.get() + 1);
                        div().h(px(20.0)).child("row")
                    })),
            )
    }
}

fn bench(loading: bool, cx: &mut TestAppContext) -> (gpui::Entity<Bench>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    let (view, cx) = cx.add_window_view(|_, _| Bench {
        loading,
        presses: 0,
        built: Rc::new(Cell::new(0)),
    });
    cx.run_until_parked();
    (view, cx)
}

fn press_box(cx: &mut VisualTestContext) {
    let at = point(px(100.0), px(50.0));
    cx.simulate_mouse_move(at, None, Modifiers::none());
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn the_veil_stops_presses_only_while_loading(cx: &mut TestAppContext) {
    let (view, cx) = bench(true, cx);
    press_box(cx);
    assert_eq!(view.read_with(cx, |bench, _| bench.presses), 0);
    view.update(cx, |bench, cx| {
        bench.loading = false;
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    press_box(cx);
    assert_eq!(view.read_with(cx, |bench, _| bench.presses), 1);
}

#[gpui::test]
fn a_lazy_row_builds_once_it_scrolls_into_view(cx: &mut TestAppContext) {
    let (view, cx) = bench(false, cx);
    let built = view.read_with(cx, |bench, _| bench.built.clone());
    assert_eq!(built.get(), 0);
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(50.0), px(150.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-2000.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    for _ in 0..3 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
    assert!(built.get() >= 1, "the row was built");
}

#[test]
#[should_panic(expected = "is behind")]
fn a_buffer_behind_the_value_is_refused() {
    let _ = ProgressBar::new("bar", 0.6).buffer(0.4);
}

#[test]
#[should_panic(expected = "not 0..=1")]
fn a_value_past_whole_is_refused() {
    let _ = ProgressBar::new("bar", 1.2);
}

/// A sweeping bar at the top of a tall column, and where the next child starts.
struct Column {
    after: Rc<Cell<f32>>,
}

impl Render for Column {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let after = self.after.clone();
        div()
            .flex()
            .flex_col()
            .h(px(100.0))
            .child(ProgressBar::indeterminate("sweep"))
            .child(
                crate::primitives::Measure::new("after", move |bounds, _, _| {
                    after.set(f32::from(bounds.origin.y))
                })
                .h(px(10.0)),
            )
    }
}

#[gpui::test]
fn a_sweeping_bar_keeps_its_thickness_in_a_tall_column(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let after = Rc::new(Cell::new(-1.0));
    let (_, cx) = cx.add_window_view({
        let after = after.clone();
        move |_, _| Column { after }
    });
    cx.run_until_parked();
    let thickness = cx.update(|window, cx| {
        f32::from(
            crate::theme::ActiveTheme::theme(cx)
                .progress_thickness()
                .to_pixels(window.rem_size()),
        )
    });
    assert_eq!(after.get(), thickness, "the bar took only its thickness");
}
