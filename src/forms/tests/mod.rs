mod choices;

use gpui::{
    AppContext as _, Context, Entity, EntityInputHandler, IntoElement, KeyUpEvent, Keystroke,
    ParentElement, Render, TestAppContext, VisualTestContext, Window, div,
};

use super::{Input, MaskedInput, NumberInput, PinInput, TextInput, bind_keys};
use crate::theme::Theme;

struct Fields {
    fields: Vec<Entity<TextInput>>,
    clearable: bool,
}

impl Render for Fields {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().children(self.fields.iter().map(|field| {
            let input = Input::new(field);
            if self.clearable {
                input.clearable()
            } else {
                input
            }
        }))
    }
}

pub(super) fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        bind_keys(cx);
    });
}

fn open(
    count: usize,
    clearable: bool,
    cx: &mut TestAppContext,
) -> (Vec<Entity<TextInput>>, &mut VisualTestContext) {
    open_with(count, clearable, |input| input, cx)
}

fn open_with(
    count: usize,
    clearable: bool,
    build: impl Fn(TextInput) -> TextInput + 'static,
    cx: &mut TestAppContext,
) -> (Vec<Entity<TextInput>>, &mut VisualTestContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|window, cx| Fields {
        fields: (0..count)
            .map(|_| cx.new(|cx| build(TextInput::new(window, cx))))
            .collect(),
        clearable,
    });
    let fields = view.read_with(cx, |view, _| view.fields.clone());
    (fields, cx)
}

fn text(field: &Entity<TextInput>, cx: &mut VisualTestContext) -> String {
    field.read_with(cx, |input, _| input.text().to_string())
}

fn focus(field: &Entity<TextInput>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.focus(&field.read(cx).focus().clone()));
}

#[gpui::test]
fn tab_reaches_fields_and_skips_disabled_ones(cx: &mut TestAppContext) {
    let (fields, cx) = open(3, false, cx);
    fields[1].update(cx, |input, cx| input.set_disabled(true, cx));
    cx.run_until_parked();
    let mut order = Vec::new();
    for _ in 0..2 {
        order.push(cx.update(|window, cx| {
            window.focus_next();
            fields
                .iter()
                .position(|field| field.read(cx).focus().is_focused(window))
        }));
    }
    assert_eq!(order, [Some(0), Some(2)]);
}

#[gpui::test]
fn enter_on_a_clear_button_clears_its_own_field(cx: &mut TestAppContext) {
    let (fields, cx) = open(2, true, cx);
    for field in &fields {
        field.update(cx, |input, cx| input.set_text("kept", cx));
    }
    cx.run_until_parked();
    cx.update(|window, _| {
        window.focus_next();
        window.focus_next();
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    assert_eq!([text(&fields[0], cx), text(&fields[1], cx)], ["", "kept"]);
}

#[gpui::test]
fn a_composition_undoes_in_one_step(cx: &mut TestAppContext) {
    let (fields, cx) = open(1, false, cx);
    let field = &fields[0];
    focus(field, cx);
    cx.update(|window, cx| {
        field.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "n", None, window, cx);
            input.replace_and_mark_text_in_range(None, "ni", None, window, cx);
            input.replace_text_in_range(None, "你", window, cx);
        })
    });
    assert_eq!(text(field, cx), "你");
    cx.simulate_keystrokes("cmd-z");
    assert_eq!(text(field, cx), "");
}

#[gpui::test]
fn a_committed_composition_obeys_the_filter_and_length(cx: &mut TestAppContext) {
    let (fields, cx) = open_with(
        1,
        false,
        |input| input.filter(|ch| ch.is_ascii_digit()).max_len(1),
        cx,
    );
    let field = &fields[0];
    focus(field, cx);
    cx.update(|window, cx| {
        field.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "12a", None, window, cx);
            input.unmark_text(window, cx);
        })
    });
    assert_eq!(text(field, cx), "1");
    cx.simulate_keystrokes("cmd-z");
    assert_eq!(text(field, cx), "");
}

#[gpui::test]
fn undo_leaves_a_disabled_field_alone(cx: &mut TestAppContext) {
    let (fields, cx) = open(1, false, cx);
    let field = &fields[0];
    focus(field, cx);
    cx.simulate_input("a");
    field.update(cx, |input, cx| input.set_disabled(true, cx));
    cx.simulate_keystrokes("cmd-z");
    assert_eq!(text(field, cx), "a");
}

struct Stepped {
    value: f64,
}

impl Render for Stepped {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        NumberInput::new("stepped", self.value).on_change(move |value, _, cx| {
            view.update(cx, |view, cx| {
                view.value = value;
                cx.notify();
            })
        })
    }
}

#[gpui::test]
fn stepping_starts_from_the_typed_number(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Stepped { value: 4.0 });
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("20");
    cx.simulate_keystrokes("up");
    assert_eq!(view.read_with(cx, |view, _| view.value), 21.0);
}

struct Pin {
    code: String,
}

impl Render for Pin {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        PinInput::new("pin", 4).on_complete(move |code, _, cx| {
            let code = code.to_string();
            view.update(cx, |view, _| view.code = code);
        })
    }
}

#[gpui::test]
fn select_all_then_typing_replaces_the_pin(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Pin {
        code: String::new(),
    });
    cx.update(|window, _| window.focus_next());
    cx.simulate_input("4821");
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("1234");
    assert_eq!(view.read_with(cx, |view, _| view.code.clone()), "1234");
}

struct Masked {
    state: Entity<TextInput>,
    mask: &'static str,
}

impl Render for Masked {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        MaskedInput::new("masked", &self.state, self.mask)
    }
}

fn masked<'a>(
    mask: &'static str,
    cx: &'a mut TestAppContext,
) -> (Entity<TextInput>, &'a mut VisualTestContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|window, cx| Masked {
        state: cx.new(|cx| TextInput::new(window, cx)),
        mask,
    });
    let state = view.read_with(cx, |view, _| view.state.clone());
    focus(&state, cx);
    (state, cx)
}

#[gpui::test]
fn select_all_then_typing_refits_a_masked_field(cx: &mut TestAppContext) {
    let (state, cx) = masked("1-999", cx);
    cx.simulate_input("123");
    assert_eq!(text(&state, cx), "1-123");
    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("1");
    assert_eq!(text(&state, cx), "1-1");
}

#[gpui::test]
fn a_composition_inside_a_masked_field_fits_from_where_it_began(cx: &mut TestAppContext) {
    let (state, cx) = masked("(999) 999", cx);
    cx.simulate_input("123456");
    assert_eq!(text(&state, cx), "(123) 456");
    cx.update(|window, cx| {
        state.update(cx, |input, cx| {
            input.select(2..2, cx);
            input.replace_and_mark_text_in_range(None, "9", None, window, cx);
            input.replace_text_in_range(None, "9", window, cx);
        })
    });
    assert_eq!(text(&state, cx), "(192) 345");
}
