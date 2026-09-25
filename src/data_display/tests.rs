use std::{cell::Cell, rc::Rc};

use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Pixels, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, point, px,
};

use super::{PropertyGrid, PropertyGroup};
use crate::{primitives::Measure, theme::Theme};

/// A property grid whose drawn height the test reads.
struct Inspector(Rc<Cell<Pixels>>);

impl Render for Inspector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let height = self.0.clone();
        let editor = || div().h(px(20.0));
        div().w(px(320.0)).child(
            Measure::new("measure", move |bounds, _, _| {
                height.set(bounds.size.height)
            })
            .child(
                PropertyGrid::new("grid")
                    .group(
                        PropertyGroup::new("Layout")
                            .row("Width", editor())
                            .row("Height", editor()),
                    )
                    .group(PropertyGroup::new("Export").folded().row("Scale", editor())),
            ),
        )
    }
}

/// Frames 2ms apart, past reduced motion's 1ms folds.
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

#[gpui::test]
fn a_property_group_folds_from_its_header_and_stays_folded(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    let height = Rc::new(Cell::new(Pixels::ZERO));
    let seen = height.clone();
    let (_, cx) = cx.add_window_view(|_, _| Inspector(seen));
    settle(cx);
    let open = height.get();
    let header = point(px(4.0), px(12.0));
    cx.simulate_click(header, Modifiers::none());
    settle(cx);
    let folded = height.get();
    assert!(folded < open, "the rows fold away: {open:?} to {folded:?}");
    settle(cx);
    assert_eq!(height.get(), folded, "a new frame keeps the fold");
    cx.simulate_click(header, Modifiers::none());
    settle(cx);
    assert_eq!(height.get(), open, "a second press opens it");
}
