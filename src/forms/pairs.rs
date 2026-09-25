use std::rc::Rc;

use gpui::{
    App, Context, ElementId, Entity, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Subscription, Window, div, prelude::*,
};

use super::{Input, InputEvent, TextInput};
use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    primitives::IconName,
    theme::ControlSize,
};

type OnRows = Rc<dyn Fn(Vec<Vec<String>>, &mut Window, &mut App)>;

/// Rows of fields: `columns` wide, each row removable, more rows on demand.
struct Rows {
    columns: usize,
    rows: Vec<Vec<(Entity<TextInput>, Subscription)>>,
    placeholders: Vec<SharedString>,
    on_change: Option<OnRows>,
}

impl Rows {
    fn values(&self, cx: &App) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(field, _)| field.read(cx).text().to_string())
                    .collect()
            })
            .collect()
    }

    fn report(&self, window: &mut Window, cx: &mut Context<Self>) {
        let values = self.values(cx);
        if let Some(on_change) = self.on_change.clone() {
            on_change(values, window, cx);
        }
    }

    fn push(&mut self, seed: &[String], window: &mut Window, cx: &mut Context<Self>) {
        let row = (0..self.columns)
            .map(|column| {
                let placeholder = self.placeholders[column].clone();
                let text = seed.get(column).cloned().unwrap_or_default();
                let field = cx.new(|cx| {
                    let mut input = TextInput::new(window, cx).placeholder(placeholder);
                    input.set_text(text, cx);
                    input
                });
                let events = cx.subscribe_in(&field, window, |rows, _, event, window, cx| {
                    if *event == InputEvent::Changed {
                        rows.report(window, cx);
                    }
                });
                (field, events)
            })
            .collect();
        self.rows.push(row);
    }
}

fn rows_state(
    id: &ElementId,
    seed: &[Vec<String>],
    placeholders: &[SharedString],
    window: &mut Window,
    cx: &mut App,
) -> Entity<Rows> {
    let (seed, placeholders) = (seed.to_vec(), placeholders.to_vec());
    window.use_keyed_state(id.clone(), cx, move |window, cx: &mut Context<Rows>| {
        let mut rows = Rows {
            columns: placeholders.len(),
            rows: Vec::new(),
            placeholders,
            on_change: None,
        };
        for row in &seed {
            rows.push(row, window, cx);
        }
        rows
    })
}

fn render_rows(
    id: &ElementId,
    state: Entity<Rows>,
    add: &'static str,
    cx: &App,
) -> impl IntoElement + use<> {
    let rows = state.read(cx).rows.len();
    let lines = (0..rows).map(|ix| {
        let fields: Vec<Entity<TextInput>> = state.read(cx).rows[ix]
            .iter()
            .map(|(field, _)| field.clone())
            .collect();
        let remove = state.clone();
        div()
            .flex()
            .items_center()
            .gap_2()
            .children(fields.iter().map(|field| {
                div()
                    .flex_1()
                    .child(Input::new(field).size(ControlSize::Sm))
            }))
            .child(
                IconButton::new(("remove-row", ix), IconName::Minus)
                    .size(ControlSize::Sm)
                    .tooltip("Remove")
                    .on_click(move |_, window, cx| {
                        log::info!("rows: removed {ix}");
                        remove.update(cx, |rows, cx| {
                            rows.rows.remove(ix);
                            rows.report(window, cx);
                            cx.notify();
                        })
                    }),
            )
    });
    let grow = state.clone();
    div()
        .id(id.clone())
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .children(lines)
        .child(
            div().child(
                Button::new(("add-row", 0usize), add)
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::Plus)
                    .on_click(move |_, window, cx| {
                        log::info!("rows: added one");
                        grow.update(cx, |rows, cx| {
                            rows.push(&[], window, cx);
                            let last = rows.rows.last().expect("just pushed");
                            window.focus(&last[0].0.read(cx).focus().clone());
                            rows.report(window, cx);
                            cx.notify();
                        })
                    }),
            ),
        )
}

/// Rows of key and value fields; add and remove rows.
#[derive(IntoElement)]
pub struct KeyValueInput {
    id: ElementId,
    seed: Vec<Vec<String>>,
    on_change: Option<OnRows>,
}

impl KeyValueInput {
    /// `seed` fills the rows on first render.
    pub fn new(id: impl Into<ElementId>, seed: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            id: id.into(),
            seed: seed
                .into_iter()
                .map(|(key, value)| vec![key, value])
                .collect(),
            on_change: None,
        }
    }

    /// Reports every pair after each change.
    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<(String, String)>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(
            move |rows: Vec<Vec<String>>, window: &mut Window, cx: &mut App| {
                let pairs = rows
                    .into_iter()
                    .map(|mut row| {
                        let value = row.pop().expect("two columns");
                        (row.pop().expect("two columns"), value)
                    })
                    .collect();
                handler(pairs, window, cx)
            },
        ));
        self
    }
}

impl RenderOnce for KeyValueInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let placeholders = [SharedString::from("Key"), SharedString::from("Value")];
        let state = rows_state(&self.id, &self.seed, &placeholders, window, cx);
        state.update(cx, |rows, _| rows.on_change = self.on_change.clone());
        render_rows(&self.id, state, "Add pair", cx)
    }
}

/// A list of text entries; add and remove entries.
#[derive(IntoElement)]
pub struct ListInput {
    id: ElementId,
    seed: Vec<Vec<String>>,
    placeholder: SharedString,
    on_change: Option<OnRows>,
}

impl ListInput {
    pub fn new(id: impl Into<ElementId>, seed: impl IntoIterator<Item = String>) -> Self {
        Self {
            id: id.into(),
            seed: seed.into_iter().map(|item| vec![item]).collect(),
            placeholder: "Item".into(),
            on_change: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<String>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(
            move |rows: Vec<Vec<String>>, window: &mut Window, cx: &mut App| {
                handler(rows.into_iter().flatten().collect(), window, cx)
            },
        ));
        self
    }
}

impl RenderOnce for ListInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let placeholders = [self.placeholder.clone()];
        let state = rows_state(&self.id, &self.seed, &placeholders, window, cx);
        state.update(cx, |rows, _| rows.on_change = self.on_change.clone());
        render_rows(&self.id, state, "Add item", cx)
    }
}
