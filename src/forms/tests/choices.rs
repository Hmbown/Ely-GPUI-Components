use gpui::{
    AppContext as _, Context, Entity, IntoElement, KeyUpEvent, Keystroke, Render, SharedString,
    TestAppContext, VisualTestContext, Window,
};

use super::setup;
use crate::forms::{
    Checkbox, Choice, Combobox, MultiSelect, RadioGroup, Rating, Select, Slider, TextInput,
};

fn abc(middle_off: bool) -> [Choice; 3] {
    let middle = Choice::new("b", "B");
    [
        Choice::new("a", "A"),
        if middle_off {
            middle.disabled()
        } else {
            middle
        },
        Choice::new("c", "C"),
    ]
}

struct Radios {
    chosen: SharedString,
}

fn release(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).unwrap(),
    });
}

impl Render for Radios {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        RadioGroup::new("radios", abc(true))
            .selected(self.chosen.clone())
            .on_change(move |value, _, cx| {
                view.update(cx, |view, cx| {
                    view.chosen = value.clone();
                    cx.notify();
                })
            })
    }
}

#[gpui::test]
fn radio_arrows_skip_disabled_choices_and_wrap(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Radios { chosen: "a".into() });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("down");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "c");
    cx.simulate_keystrokes("down");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "a");
}

struct Picker {
    chosen: SharedString,
}

impl Render for Picker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Select::new("select", abc(false))
            .selected(self.chosen.clone())
            .on_change(move |value, _, cx| {
                view.update(cx, |view, cx| {
                    view.chosen = value.clone();
                    cx.notify();
                })
            })
    }
}

#[gpui::test]
fn select_opens_moves_and_picks_from_the_keyboard(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Picker { chosen: "a".into() });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("down down enter");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "b");
}

struct Level {
    value: f64,
}

impl Render for Level {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Slider::new("level", self.value).on_change(move |value, _, cx| {
            view.update(cx, |view, cx| {
                view.value = value;
                cx.notify();
            })
        })
    }
}

#[gpui::test]
fn slider_keys_step_jump_and_reach_the_end(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Level { value: 10.0 });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("right pageup");
    assert_eq!(view.read_with(cx, |view, _| view.value), 21.0);
    cx.simulate_keystrokes("end");
    assert_eq!(view.read_with(cx, |view, _| view.value), 100.0);
}

struct Agree {
    on: bool,
}

impl Render for Agree {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Checkbox::new("agree", self.on)
            .label("Agree")
            .on_change(move |on, _, cx| {
                view.update(cx, |view, cx| {
                    view.on = on;
                    cx.notify();
                })
            })
    }
}

#[gpui::test]
fn space_toggles_a_focused_checkbox(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Agree { on: false });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    assert!(view.read_with(cx, |view, _| view.on));
}

#[gpui::test]
fn a_disabled_choice_still_leaves_the_group_reachable(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Radios { chosen: "b".into() });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("down");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), "c");
}

struct Several {
    chosen: Vec<SharedString>,
}

impl Render for Several {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        MultiSelect::new("several", abc(false))
            .selected(self.chosen.clone())
            .on_change(move |next, _, cx| {
                view.update(cx, |view, cx| {
                    view.chosen = next.to_vec();
                    cx.notify();
                })
            })
    }
}

#[gpui::test]
fn a_full_enter_press_opens_a_multi_select_and_keeps_it_open(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Several { chosen: Vec::new() });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("enter");
    release("enter", cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(view.read_with(cx, |view, _| view.chosen.clone()), ["a"]);
}

struct Typed {
    state: Entity<TextInput>,
    chosen: Option<SharedString>,
}

impl Render for Typed {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let mut combo = Combobox::new(
            "typed",
            &self.state,
            [
                Choice::new("a", "A").disabled(),
                Choice::new("b", "B"),
                Choice::new("c", "C"),
            ],
        )
        .on_change(move |value, _, cx| {
            view.update(cx, |view, cx| {
                view.chosen = Some(value.clone());
                cx.notify();
            })
        });
        if let Some(value) = self.chosen.clone() {
            combo = combo.selected(value);
        }
        combo
    }
}

fn typed<'a>(
    chosen: Option<&str>,
    cx: &'a mut TestAppContext,
) -> (Entity<Typed>, &'a mut VisualTestContext) {
    setup(cx);
    let chosen = chosen.map(|value| SharedString::from(value.to_string()));
    cx.add_window_view(move |window, cx| Typed {
        state: cx.new(|cx| TextInput::new(window, cx)),
        chosen,
    })
}

#[gpui::test]
fn enter_in_a_combobox_skips_a_disabled_first_row(cx: &mut TestAppContext) {
    let (view, cx) = typed(None, cx);
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("enter");
    assert_eq!(
        view.read_with(cx, |view, _| view.chosen.clone()),
        Some("b".into())
    );
}

#[gpui::test]
fn a_combobox_shows_the_value_its_owner_sets(cx: &mut TestAppContext) {
    let (view, cx) = typed(Some("b"), cx);
    let state = view.read_with(cx, |view, _| view.state.clone());
    assert_eq!(
        state.read_with(cx, |input, _| input.text().to_string()),
        "B"
    );
    view.update(cx, |view, cx| {
        view.chosen = Some("c".into());
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        state.read_with(cx, |input, _| input.text().to_string()),
        "C"
    );
}

struct Stars {
    value: u8,
    disabled: bool,
}

impl Render for Stars {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Rating::new("stars", self.value)
            .disabled(self.disabled)
            .on_change(move |value, _, cx| {
                view.update(cx, |view, cx| {
                    view.value = value;
                    cx.notify();
                })
            })
    }
}

#[gpui::test]
fn a_disabled_rating_ignores_the_arrows(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Stars {
        value: 2,
        disabled: false,
    });
    cx.update(|window, _| window.focus_next());
    view.update(cx, |view, cx| {
        view.disabled = true;
        cx.notify();
    });
    cx.simulate_keystrokes("right");
    assert_eq!(view.read_with(cx, |view, _| view.value), 2);
}

struct Free {
    state: Entity<TextInput>,
}

impl Render for Free {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Combobox::new("free", &self.state, abc(false)).free()
    }
}

#[gpui::test]
fn a_free_combobox_keeps_the_text_it_mounts_with(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|window, cx| Free {
        state: cx.new(|cx| {
            let mut input = TextInput::new(window, cx);
            input.set_text("custom label", cx);
            input
        }),
    });
    let state = view.read_with(cx, |view, _| view.state.clone());
    assert_eq!(
        state.read_with(cx, |input, _| input.text().to_string()),
        "custom label"
    );
}
