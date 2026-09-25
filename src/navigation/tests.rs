use gpui::{
    Context, IntoElement, KeyUpEvent, Keystroke, Modifiers, ParentElement, Render, ScrollDelta,
    ScrollWheelEvent, SharedString, Styled, TestAppContext, TouchPhase, VisualTestContext, Window,
    div, point, px,
};

use super::{Breadcrumb, Crumb, EditorTab, EditorTabs, Tabs, Wizard};
use crate::{forms::Choice, theme::Theme};

fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        crate::forms::bind_keys(cx);
    });
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).unwrap(),
    });
}

struct Strip {
    chosen: SharedString,
}

impl Render for Strip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let tabs = [
            Choice::new("a", "A"),
            Choice::new("b", "B").disabled(),
            Choice::new("c", "C"),
        ];
        Tabs::new("tabs", tabs, self.chosen.clone()).on_change(move |value, _, cx| {
            view.update(cx, |view, cx| {
                view.chosen = value.clone();
                cx.notify();
            })
        })
    }
}

#[gpui::test]
fn tab_arrows_skip_disabled_tabs_and_wrap(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Strip { chosen: "a".into() });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("right");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "c");
    cx.simulate_keystrokes("right");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "a");
}

struct Path {
    picked: Option<(usize, SharedString)>,
}

impl Render for Path {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Breadcrumb::new(
            "path",
            [
                Crumb::new("home", "Home"),
                Crumb::new("docs", "Docs").siblings([
                    Choice::new("docs", "Docs"),
                    Choice::new("music", "Music"),
                    Choice::new("photos", "Photos"),
                ]),
                Crumb::new("notes", "notes.md"),
            ],
        )
        .on_select(move |level, value, _, cx| {
            let value = value.clone();
            view.update(cx, |view, _| view.picked = Some((level, value)));
        })
    }
}

#[gpui::test]
fn a_level_lists_its_siblings_and_picks_one(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Path { picked: None });
    cx.update(|window, _| {
        window.focus_next();
        window.focus_next();
    });
    press("down", cx);
    press("down", cx);
    press("enter", cx);
    let picked = view.read_with(cx, |view, _| view.picked.clone());
    assert_eq!(picked, Some((1, SharedString::from("music"))));
}

struct Flow {
    step: usize,
    done: bool,
}

impl Render for Flow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (stepper, finisher) = (cx.entity(), cx.entity());
        let steps = [
            Choice::new("account", "Account"),
            Choice::new("plan", "Plan"),
            Choice::new("confirm", "Confirm"),
        ];
        Wizard::new("flow", steps, self.step)
            .on_step(move |to, _, cx| {
                stepper.update(cx, |view, cx| {
                    view.step = to;
                    cx.notify();
                })
            })
            .on_finish(move |_, cx| finisher.update(cx, |view, _| view.done = true))
    }
}

#[gpui::test]
fn the_wizard_walks_forward_back_and_finishes(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Flow {
        step: 0,
        done: false,
    });
    cx.update(|window, _| window.focus_next());
    press("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.step), 1);
    cx.update(|window, _| {
        window.blur();
        window.focus_next();
        window.focus_next();
    });
    press("enter", cx);
    assert_eq!(view.read_with(cx, |view, _| view.step), 0);
    view.update(cx, |view, cx| {
        view.step = 2;
        cx.notify();
    });
    cx.update(|window, _| {
        window.blur();
        for _ in 0..4 {
            window.focus_next();
        }
    });
    press("enter", cx);
    assert!(view.read_with(cx, |view, _| view.done));
}

struct Docs {
    chosen: SharedString,
    clicked: Option<SharedString>,
    closed: Option<SharedString>,
}

impl Render for Docs {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (picker, closer) = (cx.entity(), cx.entity());
        let tabs = (0..10).map(|ix| EditorTab::new(format!("t{ix}"), format!("document-{ix}.rs")));
        div().w(px(200.0)).child(
            EditorTabs::new("docs", tabs)
                .selected(self.chosen.clone())
                .on_select(move |id, _, cx| {
                    let id = id.clone();
                    picker.update(cx, |view, _| view.clicked = Some(id));
                })
                .on_close(move |id, _, cx| {
                    let id = id.clone();
                    closer.update(cx, |view, _| view.closed = Some(id));
                }),
        )
    }
}

/// The test platform never runs next-frame callbacks; a refresh stands in for the display link.
fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn click_at(x: f32, cx: &mut VisualTestContext) {
    let at = point(px(x), px(16.0));
    cx.simulate_mouse_move(at, None, Modifiers::none());
    cx.simulate_click(at, Modifiers::none());
}

#[gpui::test]
fn the_chosen_tab_scrolls_in_once_and_a_manual_scroll_stays(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Docs {
        chosen: "t9".into(),
        clicked: None,
        closed: None,
    });
    settle(cx);
    let strip_end = cx
        .update(|window, _| window.viewport_size().width)
        .min(px(200.0))
        - cx.update(|_, cx| {
            crate::theme::ActiveTheme::theme(cx)
                .control_height(crate::theme::ControlSize::Sm)
                .to_pixels(px(16.0))
        })
        - px(8.0);
    click_at(f32::from(strip_end) - 16.0, cx);
    let closed = view.read_with(cx, |view, _| view.closed.clone());
    assert_eq!(
        closed,
        Some(SharedString::from("t9")),
        "the chosen tab's close button sits at the strip's end"
    );
    view.update(cx, |view, cx| {
        view.chosen = "t8".into();
        cx.notify();
    });
    settle(cx);
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(60.0), px(16.0)),
        delta: ScrollDelta::Pixels(point(px(5000.0), px(0.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    view.update(cx, |_, cx| cx.notify());
    settle(cx);
    click_at(6.0, cx);
    let clicked = view.read_with(cx, |view, _| view.clicked.clone());
    assert_eq!(
        clicked,
        Some(SharedString::from("t0")),
        "a manual scroll stays put"
    );
}
