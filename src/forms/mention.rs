use std::{ops::Range, rc::Rc};

use gpui::{
    App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::*,
};

use super::{
    Highlight, Input, TextInput,
    suggest::{Suggestion, suggestion_list},
    text::{Down, Enter, Up},
};
use crate::theme::ActiveTheme;

const SHOWN: usize = 6;

/// The trigger being typed: its byte offset and char, ending at `caret`.
pub(crate) fn active_trigger(text: &str, caret: usize, triggers: &[char]) -> Option<(usize, char)> {
    let before = &text[..caret];
    let (ix, ch) = before
        .char_indices()
        .rev()
        .take_while(|(_, ch)| !ch.is_whitespace())
        .find(|(_, ch)| triggers.contains(ch))?;
    let opens = before[..ix]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    let word = before[ix + ch.len_utf8()..]
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '_');
    (opens && word).then_some((ix, ch))
}

/// The dismissed trigger, while it is still the one being typed.
fn dismissal(dismissed: Option<usize>, active: Option<usize>) -> Option<usize> {
    dismissed.filter(|at| active == Some(*at))
}

/// Spans of `@name` and `#tag` for a highlighter.
pub(crate) fn mention_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut previous_space = true;
    let mut chars = text.char_indices().peekable();
    while let Some((ix, ch)) = chars.next() {
        if previous_space && (ch == '@' || ch == '#') {
            let mut end = ix + 1;
            while let Some(&(at, next)) = chars.peek() {
                if !(next.is_alphanumeric() || next == '_') {
                    break;
                }
                end = at + next.len_utf8();
                chars.next();
            }
            if end > ix + 1 {
                spans.push(ix..end);
            }
            previous_space = false;
            continue;
        }
        previous_space = ch.is_whitespace();
    }
    spans
}

/// Colors mentions and tags in the link tone over a faint wash.
pub fn mention_highlights(text: &str, cx: &App) -> Vec<(Range<usize>, Highlight)> {
    let link = cx.theme().colors.link;
    mention_spans(text)
        .into_iter()
        .map(|range| {
            (
                range,
                Highlight {
                    color: link,
                    background: Some(link.opacity(0.08)),
                },
            )
        })
        .collect()
}

#[derive(Default)]
struct Picking {
    highlighted: usize,
    dismissed: Option<usize>,
}

/// A field where `@` or `#` opens suggestions at the caret. Arrows choose; Enter picks.
#[derive(IntoElement)]
pub struct MentionInput {
    id: ElementId,
    state: Entity<TextInput>,
    triggers: Vec<(char, Vec<SharedString>)>,
}

impl MentionInput {
    /// Give the state `mention_highlights` as its highlighter.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextInput>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            triggers: Vec::new(),
        }
    }

    /// Offers `handles` after `trigger`, such as `@` for people or `#` for tags.
    pub fn trigger(
        mut self,
        trigger: char,
        handles: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        let handles: Vec<SharedString> = handles.into_iter().map(Into::into).collect();
        for handle in &handles {
            assert!(
                !handle.is_empty() && handle.chars().all(|ch| ch.is_alphanumeric() || ch == '_'),
                "mention handle {handle:?} must be one word"
            );
        }
        self.triggers.push((trigger, handles));
        self
    }
}

impl RenderOnce for MentionInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let picking = window.use_keyed_state(self.id.clone(), cx, |_, _| Picking::default());
        let input = self.state.read(cx);
        let (text, caret) = (input.text().to_string(), input.cursor());
        let focused = input.focus().is_focused(window);
        let marks: Vec<char> = self.triggers.iter().map(|(trigger, _)| *trigger).collect();
        let active = active_trigger(&text, caret, &marks);
        let anchor = active.and_then(|(ix, _)| input.bounds_for(ix));
        let matches: Vec<SharedString> = active
            .map(|(ix, trigger)| {
                let query = text[ix + trigger.len_utf8()..caret].to_lowercase();
                let names = &self
                    .triggers
                    .iter()
                    .find(|(mark, _)| *mark == trigger)
                    .expect("from triggers")
                    .1;
                names
                    .iter()
                    .filter(|name| name.to_lowercase().contains(&query))
                    .take(SHOWN)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let dismissed = dismissal(picking.read(cx).dismissed, active.map(|(ix, _)| ix));
        if dismissed != picking.read(cx).dismissed {
            picking.update(cx, |picking, _| picking.dismissed = dismissed);
        }
        let open =
            focused && !matches.is_empty() && active.is_some_and(|(ix, _)| dismissed != Some(ix));
        let highlighted = picking
            .read(cx)
            .highlighted
            .min(matches.len().saturating_sub(1));
        log::debug!(
            "mention input: open {open}, {} matches, caret {anchor:?}",
            matches.len()
        );
        let rows: Vec<Suggestion> = matches
            .iter()
            .map(|name| Suggestion {
                label: active.map_or(name.clone(), |(_, trigger)| {
                    SharedString::from(format!("{trigger}{name}"))
                }),
                note: None,
            })
            .collect();
        let insert = {
            let (state, matches) = (self.state.clone(), matches.clone());
            Rc::new(move |pick: usize, _: &mut Window, cx: &mut App| {
                let Some((ix, trigger)) = active else {
                    return;
                };
                let name = &matches[pick];
                log::info!("mention input: picked {trigger}{name}");
                state.update(cx, |input, cx| {
                    input.select(ix..caret, cx);
                    input.insert(&format!("{trigger}{name} "), cx);
                });
            })
        };
        let count = matches.len();
        let (up, down, enter, escape) = (
            picking.clone(),
            picking.clone(),
            insert.clone(),
            picking.clone(),
        );
        div()
            .id(self.id)
            .relative()
            .w_full()
            .capture_action(move |_: &Up, _, cx| {
                if open {
                    cx.stop_propagation();
                    up.update(cx, |picking, cx| {
                        picking.highlighted = (picking.highlighted + count - 1) % count;
                        cx.notify();
                    });
                }
            })
            .capture_action(move |_: &Down, _, cx| {
                if open {
                    cx.stop_propagation();
                    down.update(cx, |picking, cx| {
                        picking.highlighted = (picking.highlighted + 1) % count;
                        cx.notify();
                    });
                }
            })
            .capture_action(move |_: &Enter, window, cx| {
                if open {
                    cx.stop_propagation();
                    enter(highlighted, window, cx);
                }
            })
            .on_key_down(move |event, _, cx| {
                if open && event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    escape.update(cx, |picking, cx| {
                        picking.dismissed = active.map(|(ix, _)| ix);
                        cx.notify();
                    });
                }
            })
            .child(Input::new(&self.state))
            .when_some(anchor.filter(|_| open), |field, anchor| {
                field.child(suggestion_list(
                    "mentions",
                    anchor,
                    &rows,
                    highlighted,
                    insert,
                    None,
                    cx,
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{active_trigger, dismissal, mention_spans};

    #[test]
    fn triggers_open_after_space_and_close_on_space() {
        let marks = ['@', '#'];
        assert_eq!(active_trigger("hi @ad", 6, &marks), Some((3, '@')));
        assert_eq!(active_trigger("@", 1, &marks), Some((0, '@')));
        assert_eq!(active_trigger("mail@ad", 7, &marks), None);
        assert_eq!(active_trigger("hi @ada lovelace", 16, &marks), None);
        assert_eq!(active_trigger("go #rel", 7, &marks), Some((3, '#')));
    }

    #[test]
    fn a_dismissal_ends_with_its_trigger() {
        assert_eq!(dismissal(Some(0), Some(0)), Some(0));
        assert_eq!(dismissal(Some(0), None), None);
        assert_eq!(dismissal(Some(0), Some(4)), None);
    }

    #[test]
    fn spans_cover_whole_names() {
        assert_eq!(
            mention_spans("ping @ada and #release now"),
            vec![5..9, 14..22]
        );
        assert_eq!(
            mention_spans("a@b # c"),
            Vec::<std::ops::Range<usize>>::new()
        );
    }
}
