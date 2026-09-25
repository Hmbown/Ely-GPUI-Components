use gpui::{
    Context, IntoElement, Modifiers, MouseButton, Render, TestAppContext, Window, point, px,
};

use super::setup;
use crate::{
    forms::{IconPicker, SignaturePad, Stroke},
    primitives::IconName,
};

struct Icons {
    picked: Option<IconName>,
}

impl Render for Icons {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        IconPicker::new("icons").on_change(move |icon, _, cx| {
            view.update(cx, |view, cx| {
                view.picked = Some(icon);
                cx.notify();
            })
        })
    }
}

#[gpui::test]
fn an_icon_is_searched_walked_and_picked_from_the_keyboard(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Icons { picked: None });
    cx.update(|window, _| window.focus_next());
    cx.simulate_input("arrow down");
    cx.simulate_keystrokes("down right enter");
    let wanted: Vec<IconName> = IconName::ALL
        .iter()
        .copied()
        .filter(|icon| icon.name().replace('-', " ").contains("arrow down"))
        .collect();
    assert!(wanted.len() > 1, "{wanted:?}");
    assert_eq!(view.read_with(cx, |view, _| view.picked), Some(wanted[1]));
}

struct Pad {
    strokes: Vec<Stroke>,
}

impl Render for Pad {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        SignaturePad::new("pad", self.strokes.clone()).on_change(move |next, _, cx| {
            view.update(cx, |view, cx| {
                view.strokes = next.to_vec();
                cx.notify();
            })
        })
    }
}

#[gpui::test]
fn a_drag_on_the_pad_lands_as_one_stroke(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Pad {
        strokes: Vec::new(),
    });
    let none = Modifiers::none();
    cx.simulate_mouse_down(point(px(40.0), px(80.0)), MouseButton::Left, none);
    for (x, y) in [(60.0, 70.0), (90.0, 100.0), (120.0, 90.0)] {
        cx.simulate_mouse_move(point(px(x), px(y)), MouseButton::Left, none);
    }
    cx.simulate_mouse_up(point(px(120.0), px(90.0)), MouseButton::Left, none);
    let strokes = view.read_with(cx, |view, _| view.strokes.clone());
    assert_eq!(strokes.len(), 1, "{strokes:?}");
    let stroke = &strokes[0];
    assert!(stroke.len() >= 2, "{stroke:?}");
    let moved = stroke[stroke.len() - 1] - stroke[0];
    assert_eq!(moved, point(px(80.0), px(10.0)), "{stroke:?}");
}

#[gpui::test]
fn one_move_keeps_both_ends_of_the_stroke(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Pad {
        strokes: Vec::new(),
    });
    let none = Modifiers::none();
    cx.simulate_mouse_down(point(px(40.0), px(80.0)), MouseButton::Left, none);
    cx.simulate_mouse_move(point(px(100.0), px(60.0)), MouseButton::Left, none);
    cx.simulate_mouse_up(point(px(100.0), px(60.0)), MouseButton::Left, none);
    let strokes = view.read_with(cx, |view, _| view.strokes.clone());
    assert_eq!(strokes.len(), 1, "{strokes:?}");
    let stroke = &strokes[0];
    assert_eq!(stroke.len(), 2, "{stroke:?}");
    assert_eq!(stroke[1] - stroke[0], point(px(60.0), px(-20.0)));
}
