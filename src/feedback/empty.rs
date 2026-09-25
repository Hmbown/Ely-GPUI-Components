use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FontWeight, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::*,
};
use smallvec::SmallVec;

use crate::{
    motion,
    primitives::{Icon, IconName},
    theme::{ActiveTheme, IconSize, TextSize},
};

/// What a view shows with nothing in it: an icon on a soft disc, a title, a line of help and actions. It rises in when it appears.
#[derive(IntoElement)]
pub struct EmptyState {
    id: ElementId,
    icon: IconName,
    title: SharedString,
    body: Option<SharedString>,
    actions: SmallVec<[AnyElement; 2]>,
}

impl EmptyState {
    pub fn new(id: impl Into<ElementId>, icon: IconName, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            title: title.into(),
            body: None,
            actions: SmallVec::new(),
        }
    }

    pub fn body(mut self, text: impl Into<SharedString>) -> Self {
        self.body = Some(text.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .px_6()
            .py_8()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(theme.icon_size(IconSize::Xxl) + theme.icon_size(IconSize::Md))
                    .rounded_full()
                    .bg(colors.sunken)
                    .child(
                        Icon::new(self.icon)
                            .size(IconSize::Lg)
                            .color(colors.fg_subtle),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .w_full()
                    .max_w(theme.prose_width())
                    .text_center()
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Lg))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.fg)
                            .child(self.title),
                    )
                    .when_some(self.body, |text, body| {
                        text.child(
                            div()
                                .text_size(theme.text_size(TextSize::Base))
                                .text_color(colors.fg_muted)
                                .child(body),
                        )
                    }),
            )
            .when(!self.actions.is_empty(), |state| {
                state.child(div().flex().gap_2().children(self.actions))
            })
            .with_animation(
                self.id,
                Animation::new(motion::duration(motion::SLOW, cx))
                    .with_easing(motion::ease_out_cubic),
                |state, t| state.opacity(t).mt(motion::NUDGE * 2.0 * (1.0 - t)),
            )
    }
}
