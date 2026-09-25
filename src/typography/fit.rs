use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, ShapedLine,
    SharedString, StatefulInteractiveElement, Styled, TextRun, Window, canvas, div, prelude::*,
    relative,
};

use crate::{
    primitives::Tooltip,
    theme::{ActiveTheme, TextSize},
};

const LEADING: f32 = 1.4;

/// Keeps `kept` chars: half from the front, half from the back.
fn elide(chars: &[char], kept: usize) -> String {
    let head = kept.div_ceil(2);
    let tail = kept / 2;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(&chars[chars.len() - tail..]);
    out
}

fn shape(text: &str, window: &mut Window) -> ShapedLine {
    let style = window.text_style();
    let run = TextRun {
        len: text.len(),
        font: style.font(),
        color: style.color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let size = style.font_size.to_pixels(window.rem_size());
    window
        .text_system()
        .shape_line(SharedString::from(text.to_string()), size, &[run], None)
}

fn width_of(text: &str, window: &mut Window) -> Pixels {
    shape(text, window).width
}

/// Longest middle cut that fits `width`.
fn fit_middle(text: &str, width: Pixels, window: &mut Window) -> String {
    if width_of(text, window) <= width {
        return text.to_string();
    }
    if width_of("…", window) > width {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut low, mut high) = (0, chars.len());
    while low < high {
        let mid = (low + high).div_ceil(2);
        if width_of(&elide(&chars, mid), window) <= width {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    elide(&chars, low)
}

/// Shortens in the middle, keeping both ends. Paths, hashes.
#[derive(IntoElement)]
pub struct MiddleEllipsis {
    text: SharedString,
}

impl MiddleEllipsis {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for MiddleEllipsis {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = theme.text_size(TextSize::Base);
        let line = size.to_pixels(window.rem_size()) * LEADING;
        let text = self.text;
        div()
            .w_full()
            .h(line)
            .overflow_hidden()
            .text_size(size)
            .text_color(theme.colors.fg)
            .child(
                canvas(
                    move |bounds, window, _| {
                        let shown = fit_middle(&text, bounds.size.width, window);
                        shape(&shown, window)
                    },
                    move |bounds, shaped, window, cx| {
                        shaped
                            .paint(bounds.origin, line, window, cx)
                            .expect("middle ellipsis failed to paint");
                    },
                )
                .size_full(),
            )
    }
}

/// Truncates at the end; the full text shows on hover only when cut.
#[derive(IntoElement)]
pub struct EllipsisTooltip {
    id: ElementId,
    text: SharedString,
}

impl EllipsisTooltip {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

impl RenderOnce for EllipsisTooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let cut = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let is_cut = *cut.read(cx);
        let theme = cx.theme();
        let measured = self.text.clone();
        div()
            .id(self.id)
            .relative()
            .w_full()
            .truncate()
            .text_size(theme.text_size(TextSize::Base))
            .line_height(relative(LEADING))
            .text_color(theme.colors.fg)
            .child(self.text.clone())
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let natural = width_of(&measured, window);
                        let overflow = natural > bounds.size.width;
                        if *cut.read(cx) != overflow {
                            log::debug!(
                                "ellipsis tooltip: {natural:?} in {:?}, cut {overflow}",
                                bounds.size.width
                            );
                            cut.update(cx, |cut, cx| {
                                *cut = overflow;
                                cx.notify();
                                window.request_animation_frame();
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
            .when(is_cut, |el| el.tooltip(Tooltip::text(self.text)))
    }
}

#[cfg(test)]
mod tests {
    use super::elide;

    #[test]
    fn elide_keeps_both_ends() {
        let chars: Vec<char> = "abcdefghij".chars().collect();
        assert_eq!(elide(&chars, 4), "ab…ij");
        assert_eq!(elide(&chars, 5), "abc…ij");
        assert_eq!(elide(&chars, 0), "…");
        let wide: Vec<char> = "路径/很长/文件.rs".chars().collect();
        assert_eq!(elide(&wide, 4), "路径…rs");
    }
}
