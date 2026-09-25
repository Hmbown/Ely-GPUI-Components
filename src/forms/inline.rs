use std::rc::Rc;

use gpui::{
    App, Context, ElementId, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
    prelude::*,
};

use super::{Input, InputEvent, TextInput};
use crate::{
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, Radius},
};

type OnCommit = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(Default)]
struct Editing {
    field: Option<(Entity<TextInput>, Subscription)>,
    on_commit: Option<OnCommit>,
}

impl Editing {
    /// Ends editing; keeps the text unless `cancel`.
    fn finish(&mut self, cancel: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some((field, _)) = self.field.take() else {
            return;
        };
        if cancel {
            log::info!("inline edit: cancelled");
        } else {
            let text = SharedString::from(field.read(cx).text().to_string());
            log::info!("inline edit: kept");
            if let Some(commit) = self.on_commit.clone() {
                commit(&text, window, cx);
            }
        }
        cx.notify();
    }
}

/// Text that becomes a field when clicked. Enter or leaving keeps; Escape reverts.
#[derive(IntoElement)]
pub struct InlineEdit {
    id: ElementId,
    value: SharedString,
    placeholder: SharedString,
    on_commit: Option<OnCommit>,
}

impl InlineEdit {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: "Empty".into(),
            on_commit: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn on_commit(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for InlineEdit {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Editing::default());
        state.update(cx, |editing, _| editing.on_commit = self.on_commit.clone());
        if let Some(field) = state
            .read(cx)
            .field
            .as_ref()
            .map(|(field, _)| field.clone())
        {
            let escape = state.clone();
            return div()
                .id(self.id)
                .w_full()
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        escape.update(cx, |editing, cx| editing.finish(true, window, cx));
                    }
                })
                .child(Input::new(&field).size(ControlSize::Sm))
                .into_any_element();
        }
        let theme = cx.theme();
        let colors = &theme.colors;
        let empty = self.value.is_empty();
        let (value, start) = (self.value.clone(), state);
        let group = SharedString::from(format!("inline-edit-{:?}", self.id));
        div()
            .id(self.id)
            .group(group.clone())
            .flex()
            .items_center()
            .gap_2()
            .h(theme.control_height(ControlSize::Sm))
            .px(theme.control_padding(ControlSize::Sm))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(gpui::transparent_black())
            .cursor_text()
            .hover(|style| style.bg(colors.hover))
            .text_color(if empty { colors.fg_subtle } else { colors.fg })
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .on_click(move |_, window, cx| {
                log::info!("inline edit: editing");
                let value = value.to_string();
                let field = cx.new(|cx| {
                    let mut input = TextInput::new(window, cx);
                    input.set_text(value, cx);
                    input
                });
                let all = field.read(cx).text().len();
                field.update(cx, |input, cx| input.select(0..all, cx));
                window.focus(&field.read(cx).focus().clone());
                let owner = start.clone();
                let events = window.subscribe(&field, cx, move |_, event, window, cx| {
                    if matches!(event, InputEvent::Submit | InputEvent::Blur) {
                        owner.update(cx, |editing, cx| editing.finish(false, window, cx));
                    }
                });
                start.update(cx, |editing, cx| {
                    editing.field = Some((field, events));
                    cx.notify();
                });
            })
            .child(if empty { self.placeholder } else { self.value })
            .child(
                div()
                    .invisible()
                    .group_hover(group, |style| style.visible())
                    .child(
                        Icon::new(IconName::Pencil)
                            .size(IconSize::Xs)
                            .color(colors.fg_subtle),
                    ),
            )
            .into_any_element()
    }
}
