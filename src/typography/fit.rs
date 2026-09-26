use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, ShapedLine,
    SharedString, StatefulInteractiveElement, Styled, TextRun, Window, canvas, div, prelude::*,
    relative, transparent_black,
};

use crate::{
    primitives::Tooltip,
    theme::{ActiveTheme, TextSize},
};

pub(crate) const LEADING: f32 = 1.4;

/// Keeps `kept` chars: half from the front, half from the back.
fn elide(chars: &[char], kept: usize) -> String {
    let head = kept.div_ceil(2);
    let tail = kept / 2;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(&chars[chars.len() - tail..]);
    out
}

/// How wide `text` sets at `size` in the text style around it.
pub(crate) fn text_width(text: &str, size: Pixels, window: &Window) -> Pixels {
    let style = window.text_style();
    let run = TextRun {
        len: text.len(),
        font: style.font(),
        color: style.color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(SharedString::from(text.to_string()), size, &[run], None)
        .width
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

/// Longest front that fits `width` with an ellipsis after it.
fn fit_end(text: &str, width: Pixels, window: &mut Window) -> String {
    if width_of(text, window) <= width {
        return text.to_string();
    }
    if width_of("…", window) > width {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let cut = |kept: usize| {
        let front: String = chars[..kept].iter().collect();
        let mut out: String = front
            .trim_end_matches(|ch: char| ch.is_whitespace() || matches!(ch, '.' | ',' | ';' | ':'))
            .into();
        out.push('…');
        out
    };
    let (mut low, mut high) = (0, chars.len());
    while low < high {
        let mid = (low + high).div_ceil(2);
        if width_of(&cut(mid), window) <= width {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    cut(low)
}

/// One line in the text style around it, cut at the end with an ellipsis when its box is too narrow. gpui 0.2.2 keeps a line's first measure, so its own `truncate` clips instead in flex boxes.
#[derive(IntoElement)]
pub struct Ellipsis {
    text: SharedString,
}

impl Ellipsis {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for Ellipsis {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let text = self.text.clone();
        div()
            .relative()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .child(div().text_color(transparent_black()).child(self.text))
            .child(
                canvas(
                    move |bounds, window, _| {
                        shape(&fit_end(&text, bounds.size.width, window), window)
                    },
                    |bounds, shaped, window, cx| {
                        let line = window.text_style().line_height_in_pixels(window.rem_size());
                        shaped
                            .paint(bounds.origin, line, window, cx)
                            .expect("an ellipsis line paints");
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
    }
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

#[cfg(all(test, feature = "test-support"))]
mod fits {
    use gpui::{Pixels, TestAppContext};

    use super::{fit_end, width_of};

    #[gpui::test]
    fn an_ellipsis_keeps_the_longest_front_that_fits(cx: &mut TestAppContext) {
        let cx = cx.add_empty_window();
        cx.update(|window, _| {
            let text = "Good interfaces. Quiet ones";
            let whole = width_of(text, window);
            assert_eq!(
                fit_end(text, whole, window),
                text,
                "a line that fits stays whole"
            );
            let room = width_of("Good interfaces.…", window);
            assert_eq!(
                fit_end(text, room, window),
                "Good interfaces…",
                "no stop or space before the ellipsis"
            );
            assert_eq!(
                fit_end(text, Pixels::ZERO, window),
                "",
                "no room shows nothing"
            );
        });
    }
}
