use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Point, RenderOnce, Styled, Window, anchored, deferred, div,
    prelude::*,
};
use smallvec::SmallVec;

use crate::{motion, theme::ActiveTheme};

type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// Full-window scrim. Children sit centered above it.
#[derive(IntoElement)]
pub struct Backdrop {
    id: ElementId,
    on_dismiss: Option<Dismiss>,
    children: SmallVec<[AnyElement; 1]>,
}

impl Backdrop {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on_dismiss: None,
            children: SmallVec::new(),
        }
    }

    /// Runs on a click outside the children.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Backdrop {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Backdrop {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = window.viewport_size();
        let fade = motion::duration(motion::BASE, cx);
        let scrim = div()
            .id(self.id.clone())
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(cx.theme().colors.backdrop)
            .occlude()
            .when_some(self.on_dismiss, |scrim, dismiss| {
                scrim.on_mouse_down(MouseButton::Left, move |_, window, cx| dismiss(window, cx))
            })
            .with_animation(
                self.id,
                Animation::new(fade).with_easing(motion::ease_out_cubic),
                |scrim, t| scrim.opacity(t),
            );

        deferred(
            anchored().position(Point::default()).child(
                div()
                    .relative()
                    .w(viewport.width)
                    .h(viewport.height)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(scrim)
                    .children(
                        self.children
                            .into_iter()
                            .map(|child| div().relative().occlude().child(child)),
                    ),
            ),
        )
    }
}
