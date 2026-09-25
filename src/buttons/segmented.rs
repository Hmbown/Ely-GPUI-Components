use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::*,
};

use super::button::label_size;
use crate::{
    motion::{self, Axis, Marker, glide, measure_item, measure_origin, slide},
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme, ControlSize, Elevation, Radius},
};

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

impl RenderOnce for SegmentedControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let values: Vec<SharedString> = self
            .segments
            .iter()
            .map(|(value, ..)| value.clone())
            .collect();
        let chosen = values
            .iter()
            .position(|value| *value == self.selected)
            .unwrap_or_else(|| panic!("segmented control has no segment {}", self.selected));
        let (state, marker) = slide(self.id.clone(), &values, &self.selected, window, cx);
        let theme = cx.theme();
        let colors = &theme.colors;
        let (text, icon_size) = label_size(self.size);
        let duration = motion::duration(motion::SLOW, cx);
        let thumb = marker.map(
            |Marker {
                 from,
                 to,
                 generation,
             }| {
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
            },
        );
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
                    .child(measure_item(measure, ix, Axis::Horizontal))
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
            .child(measure_origin(state.clone(), Axis::Horizontal))
            .children(thumb)
            .children(segments)
    }
}
