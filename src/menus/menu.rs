use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, Div, ElementId, Entity, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Stateful, Styled,
    Window, anchored, canvas, deferred, div, prelude::*,
};

use super::model::{Entry, Kind, Menu, MenuItem};
use crate::{
    forms::{Run, float, surface},
    motion,
    primitives::{Icon, IconName, Takeover, give_back, take_focus},
    theme::{ActiveTheme, ControlSize, IconSize, Radius, TextSize},
    typography::KbdCombo,
};

/// The menu open at `depth`, following the marked rows above it; `None` once those rows changed.
fn at_depth<'a>(menu: &'a Menu, levels: &[Level], depth: usize) -> Option<&'a Menu> {
    levels.get(..depth)?.iter().try_fold(menu, |menu, level| {
        match level
            .at
            .and_then(|ix| menu.item_at(ix))
            .map(|item| &item.kind)
        {
            Some(Kind::Sub(sub)) => Some(sub),
            _ => None,
        }
    })
}

/// Fits open levels to the menu as it is now: a mark on a row gone or disabled clears, and levels under a row that no longer opens them close.
fn settle(menu: &Menu, levels: &mut Vec<Level>) {
    let mut here = menu;
    for depth in 0..levels.len() {
        let at = levels[depth]
            .at
            .filter(|&ix| here.item_at(ix).is_some_and(|item| !item.disabled));
        levels[depth].at = at;
        match at.and_then(|ix| here.item_at(ix)).map(|item| &item.kind) {
            Some(Kind::Sub(sub)) if depth + 1 < levels.len() => here = sub,
            _ => {
                levels.truncate(depth + 1);
                return;
            }
        }
    }
}

/// One open level: its marked row, that row's box and its panel's box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Level {
    at: Option<usize>,
    row: Bounds<Pixels>,
    panel: Bounds<Pixels>,
}

/// Where an open menu hangs: under its host's box, or at a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Spot {
    Under,
    At(Point<Pixels>),
}

/// A host's menu: where it hangs, its open levels root first, the host's box, and the focus to hand back.
#[derive(Default)]
pub(crate) struct Open {
    spot: Option<Spot>,
    levels: Vec<Level>,
    anchor: Bounds<Pixels>,
    takeover: Option<Entity<Takeover>>,
}

impl Open {
    pub(crate) fn is_open(&self) -> bool {
        self.spot.is_some()
    }

    /// Opens at `spot`, with the first row marked when `marked`.
    pub(crate) fn show(state: &Entity<Open>, menu: &Menu, spot: Spot, marked: bool, cx: &mut App) {
        let first = menu.step(None, 1).filter(|_| marked);
        state.update(cx, |open, cx| {
            open.spot = Some(spot);
            open.levels = vec![Level {
                at: first,
                ..Level::default()
            }];
            cx.notify();
        });
    }

    /// Closes and hands focus back to whatever held it before the menu opened.
    pub(crate) fn close(state: &Entity<Open>, window: &mut Window, cx: &mut App) {
        let takeover = state.update(cx, |open, cx| {
            open.spot = None;
            open.levels.clear();
            cx.notify();
            open.takeover.take()
        });
        if let Some(takeover) = takeover {
            give_back(&takeover, window, cx);
        }
    }
}

/// Marks row `at` of level `depth` and drops deeper levels; a submenu row opens when `open_sub`.
fn mark(state: &Entity<Open>, menu: &Menu, depth: usize, at: usize, open_sub: bool, cx: &mut App) {
    state.update(cx, |open, cx| {
        let Some(here) = at_depth(menu, &open.levels, depth) else {
            return;
        };
        let sub = matches!(here.item_at(at).map(|item| &item.kind), Some(Kind::Sub(_)));
        open.levels.truncate(depth + 1);
        if open.levels[depth].at != Some(at) {
            open.levels[depth].at = Some(at);
            open.levels[depth].row = Bounds::default();
        }
        if open_sub && sub {
            open.levels.push(Level::default());
        }
        cx.notify();
    });
}

/// Runs a row. A submenu opens with its first row marked; anything else closes the menu first.
fn choose(
    state: &Entity<Open>,
    menu: &Menu,
    depth: usize,
    at: usize,
    close: &Run,
    window: &mut Window,
    cx: &mut App,
) {
    let levels = &state.read(cx).levels;
    let Some(item) = at_depth(menu, levels, depth)
        .filter(|_| levels.len() > depth)
        .and_then(|here| here.item_at(at))
        .cloned()
    else {
        log::debug!("menu: a press from an older frame found row {at} of level {depth} gone");
        return;
    };
    if item.disabled {
        return;
    }
    if let Kind::Sub(sub) = &item.kind {
        let first = sub.step(None, 1);
        mark(state, menu, depth, at, false, cx);
        state.update(cx, |open, cx| {
            open.levels.push(Level {
                at: first,
                ..Level::default()
            });
            cx.notify();
        });
        return;
    }
    log::info!("menu: {}", item.label);
    close(window, cx);
    if let Some(on_click) = &item.on_click {
        on_click(window, cx);
    }
}

/// Arrows walk the deepest level; Right opens a submenu; Enter and Space choose; Left and Escape back out.
fn keys(
    menu: &Menu,
    state: &Entity<Open>,
    key: &str,
    close: &Run,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let levels = state.read(cx).levels.clone();
    let Some(depth) = levels.len().checked_sub(1) else {
        return false;
    };
    let Some(here) = at_depth(menu, &levels, depth) else {
        return false;
    };
    let at = levels[depth].at;
    let sub = at
        .and_then(|at| here.item_at(at))
        .is_some_and(|item| matches!(item.kind, Kind::Sub(_)));
    match (key, at) {
        ("down" | "up", _) => {
            let by = if key == "down" { 1 } else { -1 };
            if let Some(to) = here.step(at, by) {
                mark(state, menu, depth, to, false, cx);
            }
        }
        ("right", Some(at)) if sub => choose(state, menu, depth, at, close, window, cx),
        ("enter" | "space", Some(at)) => choose(state, menu, depth, at, close, window, cx),
        ("left" | "escape", _) if depth > 0 => state.update(cx, |open, cx| {
            open.levels.pop();
            cx.notify();
        }),
        ("escape", _) => close(window, cx),
        _ => return false,
    }
    true
}

/// Records a box into the host's state: the host's own, a marked row's or a panel's.
fn measure(
    state: Entity<Open>,
    write: impl Fn(&mut Open, Bounds<Pixels>) -> bool + 'static,
) -> impl IntoElement {
    canvas(
        move |bounds, _, cx| {
            if state.update(cx, |open, _| write(open, bounds)) {
                state.update(cx, |_, cx| cx.notify());
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Measures the host's box, which a dropdown hangs under.
pub(crate) fn measure_host(state: Entity<Open>) -> impl IntoElement {
    measure(state, |open, bounds| {
        let changed = open.anchor != bounds;
        open.anchor = bounds;
        changed
    })
}

fn row(item: &MenuItem, ix: usize, marked: bool, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let fg = if item.disabled {
        colors.fg_disabled
    } else {
        colors.fg
    };
    let (lead, lead_color) = match &item.kind {
        Kind::Check(true) => (Some(IconName::Check), fg),
        Kind::Radio(true) => (Some(IconName::Dot), fg),
        _ => (item.icon, if item.disabled { fg } else { colors.fg_muted }),
    };
    div()
        .id(("row", ix))
        .relative()
        .flex()
        .items_center()
        .gap_2()
        .h(theme.control_height(ControlSize::Md))
        .px_2()
        .rounded(theme.radius(Radius::Md))
        .text_color(fg)
        .when(marked && !item.disabled, |row| row.bg(colors.hover))
        .when(!item.disabled, |row| row.cursor_pointer())
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(theme.icon_size(IconSize::Sm))
                .when_some(lead, |slot, icon| {
                    slot.child(Icon::new(icon).size(IconSize::Sm).color(lead_color))
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .child(item.label.clone()),
        )
        .when_some(item.keys.clone(), |row, keys| {
            row.child(
                div()
                    .flex_none()
                    .pl_4()
                    .text_color(colors.fg_subtle)
                    .child(KbdCombo::new(&keys)),
            )
        })
        .when(matches!(item.kind, Kind::Sub(_)), |row| {
            row.child(
                Icon::new(IconName::ChevronRight)
                    .size(IconSize::Xs)
                    .color(colors.fg_subtle),
            )
        })
}

fn panel(
    id: &ElementId,
    menu: &Menu,
    state: &Entity<Open>,
    depth: usize,
    close: &Run,
    cx: &App,
) -> Stateful<Div> {
    let open = state.read(cx);
    let here = at_depth(menu, &open.levels, depth).expect("settled levels follow submenu rows");
    let at = open.levels[depth].at;
    let theme = cx.theme();
    let colors = &theme.colors;
    let rows = here
        .entries
        .iter()
        .enumerate()
        .map(|(ix, entry)| match entry {
            Entry::Separator => div().h_px().my_1().bg(colors.border).into_any_element(),
            Entry::Heading(title) => div()
                .px_2()
                .pt_1p5()
                .pb_1()
                .text_size(theme.text_size(TextSize::Xs))
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.fg_subtle)
                .child(title.clone())
                .into_any_element(),
            Entry::Item(item) => {
                let marked = at == Some(ix);
                let sub = matches!(item.kind, Kind::Sub(_));
                let (hover, click, close, menu) =
                    (state.clone(), state.clone(), close.clone(), menu.clone());
                let hover_menu = menu.clone();
                row(item, ix, marked, cx)
                    .when(!item.disabled, |row| {
                        row.on_mouse_move(move |_, _, cx| {
                            let open = hover.read(cx);
                            let Some(level) = open.levels.get(depth) else {
                                return;
                            };
                            let shown = open.levels.len() > depth + 1;
                            if level.at != Some(ix) || (sub && !shown) {
                                mark(&hover, &hover_menu, depth, ix, true, cx);
                            }
                        })
                        .on_mouse_down(
                            MouseButton::Left,
                            move |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                                choose(&click, &menu, depth, ix, &close, window, cx);
                            },
                        )
                    })
                    .when(marked && sub, |row| {
                        row.child(measure(state.clone(), move |open, bounds| {
                            let level = &mut open.levels[depth];
                            let changed = level.row != bounds;
                            level.row = bounds;
                            changed
                        }))
                    })
                    .into_any_element()
            }
        });
    surface((id.clone(), format!("panel-{depth}")), cx)
        .relative()
        .min_w(theme.menu_width())
        .p_1()
        .flex()
        .flex_col()
        .children(rows)
        .child(measure(state.clone(), move |open, bounds| {
            let level = &mut open.levels[depth];
            let changed = level.panel != bounds;
            level.panel = bounds;
            changed
        }))
}

/// The open menu of host `id`, placed at its spot. It holds focus while open and gives it back on every close.
pub(crate) fn hang(
    id: &ElementId,
    menu: &Menu,
    state: &Entity<Open>,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let spot = state.read(cx).spot?;
    let takeover = take_focus((id.clone(), "takeover"), window, cx);
    let focus = takeover.read(cx).focus.clone();
    if state.read(cx).takeover.is_none() {
        state.update(cx, |open, _| open.takeover = Some(takeover));
    }
    let close: Run = {
        let (state, id) = (state.clone(), id.clone());
        Rc::new(move |window, cx| {
            log::info!("menu {id:?}: closed");
            Open::close(&state, window, cx);
        })
    };
    if !focus.is_focused(window) {
        log::info!("menu {id:?}: focus left");
        close(window, cx);
        return None;
    }
    let mut levels = state.read(cx).levels.clone();
    settle(menu, &mut levels);
    if levels != state.read(cx).levels {
        log::info!("menu {id:?}: rows changed while open");
        state.update(cx, |open, _| open.levels = levels);
    }
    let depth = state.read(cx).levels.len();
    let enter =
        Animation::new(motion::duration(motion::FAST, cx)).with_easing(motion::ease_out_cubic);
    let (keyed, key_menu, key_close) = (state.clone(), menu.clone(), close.clone());
    let (lifted, lift_menu, lift_close) = (state.clone(), menu.clone(), close.clone());
    let (out, out_close) = (state.clone(), close.clone());
    let root = panel(id, menu, state, 0, &close, cx)
        .track_focus(&focus)
        .on_key_down(move |event, window, cx| {
            let stroke = &event.keystroke;
            let key = stroke.key.as_str();
            if matches!(key, "enter" | "space") {
                if !stroke.modifiers.modified() {
                    cx.stop_propagation();
                }
                return;
            }
            if keys(&key_menu, &keyed, key, &key_close, window, cx) {
                cx.stop_propagation();
            }
        })
        .on_key_up(move |event, window, cx| {
            let stroke = &event.keystroke;
            let key = stroke.key.as_str();
            if matches!(key, "enter" | "space")
                && !stroke.modifiers.modified()
                && keys(&lift_menu, &lifted, key, &lift_close, window, cx)
            {
                cx.stop_propagation();
            }
        })
        .on_mouse_down_out(move |event, window, cx| {
            let open = out.read(cx);
            let on_host = open.spot == Some(Spot::Under) && open.anchor.contains(&event.position);
            if !on_host
                && !open
                    .levels
                    .iter()
                    .any(|level| level.panel.contains(&event.position))
            {
                out_close(window, cx);
            }
        });
    let placed = match spot {
        Spot::Under => float(state.read(cx).anchor, menu.entries.len(), root, window, cx),
        Spot::At(point) => deferred(anchored().position(point).snap_to_window().child(
            root.with_animation((id.clone(), "in"), enter.clone(), |root, t| root.opacity(t)),
        ))
        .with_priority(1)
        .into_any_element(),
    };
    let subs = (1..depth).filter_map(|level| {
        let row = state.read(cx).levels[level - 1].row;
        (row != Bounds::default()).then(|| {
            deferred(
                anchored().position(row.top_right()).snap_to_window().child(
                    panel(id, menu, state, level, &close, cx)
                        .mt_neg_1()
                        .with_animation(
                            (id.clone(), format!("sub-{level}")),
                            enter.clone(),
                            |panel, t| panel.opacity(t).ml(motion::NUDGE * (t - 1.0)),
                        ),
                ),
            )
            .with_priority(2)
            .into_any_element()
        })
    });
    Some(
        div()
            .child(placed)
            .children(subs.collect::<Vec<_>>())
            .into_any_element(),
    )
}
