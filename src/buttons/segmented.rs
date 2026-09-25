use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, App, Bounds, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, canvas, div, prelude::*,
};

use super::button::label_size;
use crate::{
    motion,
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme, ControlSize, Elevation, Radius},
};

/// Left edge and width of a segment, inside the control.
type Span = (Pixels, Pixels);

#[derive(Default)]
struct Slide {
    origin: Pixels,
    spans: Vec<Option<Span>>,
    from: Option<Span>,
    last: Option<SharedString>,
    generation: u64,
}

type OnChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Mutually exclusive segments. A thumb slides to the chosen one.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    segments: Vec<(SharedString, SharedString, Option<IconName>)>,
    selected: SharedString,
    size: ControlSize,
    on_change: Option<OnChange>,
}

impl SegmentedControl {
    pub fn new(id: impl Into<ElementId>, selected: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            segments: Vec::new(),
            selected: selected.into(),
            size: ControlSize::default(),
            on_change: None,
        }
    }

    pub fn segment(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: Option<IconName>,
    ) -> Self {
        self.segments.push((value.into(), label.into(), icon));
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

/// Position along a slide from `from` to `to`, overshooting a little.
fn glide(from: Span, to: Span, t: f32) -> Span {
    let s = motion::spring(t);
    let lerp = |a: Pixels, b: Pixels| a + (b - a) * s;
    (lerp(from.0, to.0), lerp(from.1, to.1))
}

impl RenderOnce for SegmentedControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let chosen = self
            .segments
            .iter()
            .position(|(value, ..)| *value == self.selected)
            .unwrap_or_else(|| panic!("segmented control has no segment {}", self.selected));
        let count = self.segments.len();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Slide::default());
        if state.read(cx).spans.len() != count {
            state.update(cx, |slide, _| slide.spans = vec![None; count]);
        }
        if state.read(cx).last.as_ref() != Some(&self.selected) {
            state.update(cx, |slide, _| {
                let previous = slide
                    .last
                    .as_ref()
                    .and_then(|last| self.segments.iter().position(|(value, ..)| value == last));
                slide.from = previous.and_then(|ix| slide.spans[ix]);
                slide.last = Some(self.selected.clone());
                slide.generation += 1;
            });
        }
        let (to, from, generation) = {
            let slide = state.read(cx);
            (slide.spans[chosen], slide.from, slide.generation)
        };
        let theme = cx.theme();
        let colors = &theme.colors;
        let (text, icon_size) = label_size(self.size);
        let duration = motion::duration(motion::SLOW, cx);
        let thumb = to.map(|to| {
            let from = from.unwrap_or(to);
            div()
                .absolute()
                .top_0p5()
                .bottom_0p5()
                .rounded(theme.radius(Radius::Md))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.border)
                .shadow(theme.elevation(Elevation::Raised))
                .with_animation(
                    ("segment-thumb", generation),
                    Animation::new(duration),
                    move |thumb, t| {
                        let (left, width) = glide(from, to, t);
                        thumb.left(left).w(width)
                    },
                )
        });
        let origin_state = state.clone();
        let segments = self
            .segments
            .into_iter()
            .enumerate()
            .map(|(ix, (value, label, icon))| {
                let on = ix == chosen;
                let fg = if on { colors.fg } else { colors.fg_muted };
                let (change, measure) = (self.on_change.clone(), state.clone());
                div()
                    .id(("segment", ix))
                    .relative()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_1p5()
                    .h(theme.control_height(self.size))
                    .px(theme.control_padding(self.size))
                    .rounded(theme.radius(Radius::Md))
                    .border_1()
                    .border_color(gpui::transparent_black())
                    .text_size(theme.text_size(text))
                    .font_weight(if on {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(fg)
                    .cursor_pointer()
                    .tab_index(0)
                    .focus_ring(cx)
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(move |_, window, cx| {
                        if on {
                            return;
                        }
                        log::info!("segmented control: {value}");
                        if let Some(change) = &change {
                            change(&value, window, cx);
                        }
                    })
                    .when_some(icon, |segment, icon| {
                        segment.child(Icon::new(icon).size(icon_size).color(fg))
                    })
                    .child(label)
                    .child(
                        canvas(
                            move |bounds: Bounds<Pixels>, _, cx| {
                                let span = Some((
                                    bounds.left() - measure.read(cx).origin,
                                    bounds.size.width,
                                ));
                                if measure.read(cx).spans.get(ix) != Some(&span) {
                                    measure.update(cx, |slide, cx| {
                                        slide.spans[ix] = span;
                                        cx.notify();
                                    });
                                }
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    )
            });
        div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .p_0p5()
            .rounded(theme.radius(Radius::Lg))
            .bg(colors.sunken)
            .border_1()
            .border_color(colors.border)
            .child(
                canvas(
                    move |bounds: Bounds<Pixels>, _, cx| {
                        if origin_state.read(cx).origin != bounds.left() {
                            origin_state.update(cx, |slide, cx| {
                                slide.origin = bounds.left();
                                cx.notify();
                            });
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .children(thumb)
            .children(segments)
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::glide;

    #[test]
    fn glide_starts_at_from_and_settles_on_to() {
        let (from, to) = ((px(0.0), px(40.0)), (px(100.0), px(60.0)));
        assert_eq!(glide(from, to, 0.0), from);
        let (left, width) = glide(from, to, 1.0);
        assert!((f32::from(left) - 100.0).abs() < 0.5 && (f32::from(width) - 60.0).abs() < 0.5);
    }
}
