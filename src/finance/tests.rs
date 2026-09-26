use std::rc::Rc;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
    Window, div, px,
};
use jiff::Timestamp;

use crate::finance::{CandlestickChart, ChartSync, candles::Candle, stage::Visible};
use crate::theme::Theme;

fn candles(count: usize) -> Rc<Vec<Candle>> {
    Rc::new(
        (0..count)
            .map(|ix| {
                let time = Timestamp::from_second(ix as i64 * 86_400).expect("a day");
                Candle::new(time, (10.0, 11.0, 9.0, 10.0 + ix as f64 * 0.01), 100.0)
            })
            .collect(),
    )
}

struct Pair {
    candles: Rc<Vec<Candle>>,
    sync: Entity<ChartSync>,
}

impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(800.))
            .child(CandlestickChart::new("first", self.candles.clone()).sync(&self.sync))
            .child(CandlestickChart::new("second", self.candles.clone()).sync(&self.sync))
    }
}

#[gpui::test]
fn synced_charts_move_their_shared_window_once_per_append(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let sync = cx.new(|_| ChartSync::default());
    let shared = sync.clone();
    let (view, cx) = cx.add_window_view(|_, _| Pair {
        candles: candles(40),
        sync: shared,
    });
    cx.run_until_parked();
    sync.update(cx, |sync, cx| {
        sync.visible = Some(Visible {
            start: 20.0,
            count: 20.0,
        });
        cx.notify();
    });
    view.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    view.update(cx, |pair, cx| {
        pair.candles = candles(45);
        cx.notify();
    });
    cx.run_until_parked();
    let start = sync.read_with(cx, |sync, _| sync.visible.expect("held").start);
    assert_eq!(
        start, 25.0,
        "five new candles move the shared window five, not ten"
    );
}
