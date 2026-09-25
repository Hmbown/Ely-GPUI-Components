use gpui::{
    AnyElement, App, Div, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StyleRefinement, Styled, Window, actions, div,
};

use crate::theme::ActiveTheme;

actions!(ely, [FocusNext, FocusPrev]);

/// Focus-colored border while focused. Give the element a 1px border.
pub trait FocusRing: InteractiveElement + Sized {
    fn focus_ring(self, cx: &App) -> Self {
        let ring = cx.theme().colors.focus;
        self.focus(move |style| style.border_color(ring))
    }
}

impl<E: InteractiveElement> FocusRing for E {}

/// Owns Tab and Shift-Tab for its subtree. `trap` keeps focus inside.
#[derive(IntoElement)]
pub struct FocusScope {
    base: Div,
    handle: FocusHandle,
    trap: bool,
}

impl FocusScope {
    pub fn new(handle: &FocusHandle) -> Self {
        Self {
            base: div().track_focus(handle),
            handle: handle.clone(),
            trap: false,
        }
    }

    pub fn trap(mut self) -> Self {
        self.trap = true;
        self
    }
}

impl Styled for FocusScope {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for FocusScope {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl RenderOnce for FocusScope {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (next, prev) = (self.handle.clone(), self.handle);
        let trap = self.trap;
        self.base
            .on_action(move |_: &FocusNext, window, cx| step(&next, trap, true, window, cx))
            .on_action(move |_: &FocusPrev, window, cx| step(&prev, trap, false, window, cx))
    }
}

fn step(scope: &FocusHandle, trap: bool, forward: bool, window: &mut Window, cx: &mut App) {
    let advance = |window: &mut Window| {
        if forward {
            window.focus_next()
        } else {
            window.focus_prev()
        }
    };
    if !trap {
        advance(window);
        return;
    }
    let origin = window.focused(cx);
    let mut first = None;
    loop {
        advance(window);
        let focused = window.focused(cx);
        if scope.contains_focused(window, cx) && focused.as_ref() != Some(scope) {
            return;
        }
        if focused.is_none() || focused == origin || (first.is_some() && focused == first) {
            break;
        }
        if first.is_none() {
            first = focused;
        }
    }
    log::warn!("focus scope: trap holds no tab stop; focus stays");
    if let Some(origin) = origin {
        window.focus(&origin);
    }
}

/// Focus an overlay holds while open, and what it held before.
pub(crate) struct Takeover {
    pub focus: FocusHandle,
    previous: Option<FocusHandle>,
    taken: bool,
}

/// Keyed focus for an overlay; focused on its first render.
pub(crate) fn take_focus(
    key: impl Into<gpui::ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Takeover> {
    let state = window.use_keyed_state(key, cx, |_, cx| Takeover {
        focus: cx.focus_handle(),
        previous: None,
        taken: false,
    });
    if !state.read(cx).taken {
        let previous = window.focused(cx);
        window.focus(&state.read(cx).focus.clone());
        state.update(cx, |takeover, _| {
            takeover.previous = previous;
            takeover.taken = true;
        });
    }
    state
}

/// Returns focus to whatever held it before the overlay opened.
pub(crate) fn give_back(state: &Entity<Takeover>, window: &mut Window, cx: &App) {
    match state.read(cx).previous.clone() {
        Some(previous) => {
            window.focus(&previous);
            log::info!("focus: handed back after an overlay");
        }
        None => log::info!("focus: overlay closed, nothing was focused before"),
    }
}

/// A focus handle kept for `id`, a Tab stop while enabled.
pub(crate) fn tab_stop(
    id: ElementId,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
        .tab_stop(enabled)
}
