use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, Entity, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    UniformListScrollHandle, Window, div, prelude::*, uniform_list,
};

use super::{
    Aggregate, Cell, Column,
    body::{Body, Select, sized},
    model::{figure, filtered, page, range, sorted},
};
use crate::{
    forms::{CheckState, check_mark},
    lists::{Pick, picked},
    navigation::Pagination,
    primitives::{Icon, IconName},
    theme::{ActiveTheme, Density, IconSize, TextSize},
    typography::{Caption, format},
};

/// One row: its key, which a selection names it by, and a cell for each column.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub(crate) key: SharedString,
    pub(crate) cells: Vec<Cell>,
}

impl Row {
    pub fn new(key: impl Into<SharedString>, cells: impl IntoIterator<Item = Cell>) -> Self {
        Self {
            key: key.into(),
            cells: cells.into_iter().collect(),
        }
    }
}

/// The sort, the page, where a Shift range starts, and the scroll of a long table.
#[derive(Default)]
struct View {
    sort: Option<(usize, bool)>,
    page: usize,
    anchor: usize,
    scroll: UniformListScrollHandle,
}

type OnSelect = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
/// Rows under headers that sort, rising then falling then as given. A query keeps rows holding it; pages split long results, or a long table draws only the rows in view. With a selection, a box leads each row: a press picks one, Cmd adds, Shift takes a range, the header box takes all. Footers show figures; tinted columns show where each number sits.
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    base: Div,
    columns: Vec<Column>,
    rows: Rc<Vec<Row>>,
    query: SharedString,
    page_size: Option<usize>,
    virtualized: bool,
    selected: Option<Vec<SharedString>>,
    on_select: Option<OnSelect>,
    density: Density,
}

impl DataTable {
    pub fn new(id: impl Into<ElementId>, columns: impl IntoIterator<Item = Column>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            columns: columns.into_iter().collect(),
            rows: Rc::default(),
            query: SharedString::default(),
            page_size: None,
            virtualized: false,
            selected: None,
            on_select: None,
            density: Density::Standard,
        }
    }

    /// The rows, owned or shared; a shared list is not copied.
    pub fn rows(mut self, rows: impl Into<Rc<Vec<Row>>>) -> Self {
        self.rows = rows.into();
        self
    }

    /// Keeps the rows holding this text in any cell.
    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = query.into();
        self
    }

    /// Splits the rows into pages of `size`.
    pub fn paged(mut self, size: usize) -> Self {
        assert!(size > 0, "a page holds at least a row");
        self.page_size = Some(size);
        self
    }

    /// Draws only the rows in view, for long tables. Give it a height.
    pub fn virtualized(mut self) -> Self {
        self.virtualized = true;
        self
    }

    /// Shows a box on each row; `keys` are the rows selected.
    pub fn selected(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
}

impl Styled for DataTable {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

/// The next sort after a press on column `col`: rising, then falling, then none.
fn next_sort(sort: Option<(usize, bool)>, col: usize) -> Option<(usize, bool)> {
    match sort {
        Some((was, true)) if was == col => Some((col, false)),
        Some((was, false)) if was == col => None,
        _ => Some((col, true)),
    }
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let count = self.columns.len();
        for row in self.rows.iter() {
            assert_eq!(
                row.cells.len(),
                count,
                "row {} needs a cell per column",
                row.key
            );
        }
        let view: Entity<View> =
            window.use_keyed_state((id.clone(), "view"), cx, |_, _| View::default());
        let (sort, page_at, scroll) = {
            let view = view.read(cx);
            (view.sort, view.page, view.scroll.clone())
        };
        let rows = self.rows;
        let keys: Rc<Vec<SharedString>> = Rc::new(rows.iter().map(|row| row.key.clone()).collect());
        let mut order = filtered(&rows, &self.query);
        if let Some((col, rising)) = sort {
            order = sorted(&rows, order, col, rising);
        }
        let total = order.len();
        let pages = self.page_size.map(|size| total.div_ceil(size).max(1));
        let page_at = pages.map_or(0, |pages| page_at.min(pages - 1));
        let (shown, offset) = match self.page_size {
            Some(size) => (page(&order, page_at, size).to_vec(), page_at * size),
            None => (order.clone(), 0),
        };
        let columns = Rc::new(self.columns);
        let ranges: Rc<Vec<_>> = Rc::new(
            columns
                .iter()
                .enumerate()
                .map(|(col, column)| column.scale.then(|| range(&rows, col)).flatten())
                .collect(),
        );
        let ordered: Rc<Vec<SharedString>> =
            Rc::new(order.iter().map(|ix| keys[*ix].clone()).collect());
        let selected = self.selected.map(Rc::new);
        let select: Option<Select> = selected.clone().map(|selected| {
            let (view, ordered, on_select, id) = (
                view.clone(),
                ordered.clone(),
                self.on_select.clone(),
                id.clone(),
            );
            Rc::new(
                move |position: usize, how: Pick, window: &mut Window, cx: &mut App| {
                    let next = picked(&ordered, &selected, view.read(cx).anchor, position, how);
                    view.update(cx, |view, _| {
                        if how != Pick::Range {
                            view.anchor = position;
                        }
                    });
                    log::info!("data table {id:?}: {} selected", next.len());
                    if let Some(on_select) = &on_select {
                        on_select(&next, window, cx);
                    }
                },
            ) as Select
        });
        let theme = cx.theme();
        let colors = &theme.colors;
        let height = theme.table_row(self.density);
        let check = theme.control_height(crate::theme::ControlSize::Md);
        let all = selected.as_ref().map(|selected| {
            let on = ordered.iter().filter(|key| selected.contains(key)).count();
            match on {
                0 => CheckState::Off,
                on if on == ordered.len() => CheckState::On,
                _ => CheckState::Mixed,
            }
        });
        let header = div()
            .flex()
            .items_center()
            .h(height)
            .border_b_1()
            .border_color(colors.border)
            .text_size(theme.text_size(TextSize::Xs))
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors.fg_muted)
            .when_some(all.zip(selected.clone()), |header, (state, selected)| {
                let (ordered, on_select) = (ordered.clone(), self.on_select.clone());
                header.child(
                    div()
                        .id((id.clone(), "all"))
                        .flex_none()
                        .w(check)
                        .flex()
                        .justify_center()
                        .cursor_pointer()
                        .on_click(move |_, window, cx| {
                            let next: Vec<SharedString> = if state == CheckState::On {
                                selected
                                    .iter()
                                    .filter(|key| !ordered.contains(key))
                                    .cloned()
                                    .collect()
                            } else {
                                let mut next = (*selected).clone();
                                next.extend(
                                    ordered
                                        .iter()
                                        .filter(|key| !selected.contains(key))
                                        .cloned(),
                                );
                                next
                            };
                            if let Some(on_select) = &on_select {
                                on_select(&next, window, cx);
                            }
                        })
                        .child(check_mark(state, false, false, 0, cx)),
                )
            })
            .children(columns.iter().enumerate().map(|(col, column)| {
                let here = sort
                    .filter(|(sorted, _)| *sorted == col)
                    .map(|(_, rising)| rising);
                let (view, id) = (view.clone(), id.clone());
                sized(
                    div()
                        .id((self.id.clone(), format!("head-{}", column.key)))
                        .h_full()
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_3(),
                    column,
                )
                .when(column.sortable, |head| {
                    head.cursor_pointer()
                        .hover(|style| style.text_color(colors.fg))
                        .on_click(move |_, _, cx| {
                            view.update(cx, |view, cx| {
                                view.sort = next_sort(view.sort, col);
                                view.page = 0;
                                log::info!("data table {id:?}: sort {:?}", view.sort);
                                cx.notify();
                            })
                        })
                })
                .child(column.title.clone())
                .children(here.map(|rising| {
                    Icon::new(if rising {
                        IconName::ArrowUp
                    } else {
                        IconName::ArrowDown
                    })
                    .size(IconSize::Xs)
                    .color(colors.fg_muted)
                }))
            }));
        let figures = columns
            .iter()
            .any(|column| column.aggregate.is_some())
            .then(|| {
                div()
                    .flex()
                    .items_center()
                    .h(height)
                    .text_size(theme.text_size(TextSize::Sm))
                    .font_weight(FontWeight::MEDIUM)
                    .when(selected.is_some(), |footer| {
                        footer.child(div().flex_none().w(check))
                    })
                    .children(columns.iter().enumerate().map(|(col, column)| {
                        let shown = column.aggregate.map(|how| {
                            let shares = rows
                                .first()
                                .is_some_and(|row| matches!(row.cells[col], Cell::Progress(_)));
                            let value =
                                figure(&rows, &order, col, how).map_or("—".to_string(), |value| {
                                    match how {
                                        Aggregate::Count => format!("{}", value as usize),
                                        _ if shares => format::percent(value, 0, false),
                                        _ => column.reads(value),
                                    }
                                });
                            div()
                                .flex()
                                .items_baseline()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(theme.text_size(TextSize::Xs))
                                        .text_color(colors.fg_subtle)
                                        .child(how.label()),
                                )
                                .child(value)
                        });
                        sized(div().px_3().flex().items_center(), column).children(shown)
                    }))
            });
        let body = Body {
            id: id.clone(),
            columns: columns.clone(),
            rows,
            keys,
            shown: Rc::new(shown),
            offset,
            selected,
            select,
            ranges,
            height,
            check,
        };
        let rows: AnyElement = if total == 0 {
            div()
                .flex()
                .justify_center()
                .py_6()
                .text_size(theme.text_size(TextSize::Sm))
                .text_color(colors.fg_subtle)
                .child("Nothing matches.")
                .into_any_element()
        } else if self.virtualized {
            let count = body.shown.len();
            uniform_list((id.clone(), "rows"), count, move |range, _, cx| {
                range.map(|at| body.row(at, cx)).collect()
            })
            .track_scroll(scroll)
            .flex_1()
            .min_h_0()
            .into_any_element()
        } else {
            div()
                .children((0..body.shown.len()).map(|at| body.row(at, cx)))
                .into_any_element()
        };
        let pager = pages.filter(|pages| *pages > 1).map(|pages| {
            let size = self.page_size.expect("pages come from a page size");
            let (first, last) = (page_at * size + 1, ((page_at + 1) * size).min(total));
            let (view, id) = (view.clone(), id.clone());
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt_3()
                .child(Caption::new(format!("{first}–{last} of {total}")))
                .child(
                    Pagination::new((id.clone(), "pages"), page_at + 1, pages).on_change(
                        move |page, _, cx| {
                            view.update(cx, |view, cx| {
                                view.page = page - 1;
                                log::info!("data table {id:?}: page {page}");
                                cx.notify();
                            })
                        },
                    ),
                )
        });
        self.base
            .flex()
            .flex_col()
            .child(header)
            .child(rows)
            .children(figures)
            .children(pager)
    }
}

#[cfg(test)]
mod tests {
    use super::next_sort;

    #[test]
    fn a_header_sorts_rising_then_falling_then_not() {
        assert_eq!(next_sort(None, 2), Some((2, true)));
        assert_eq!(next_sort(Some((2, true)), 2), Some((2, false)));
        assert_eq!(next_sort(Some((2, false)), 2), None);
        assert_eq!(next_sort(Some((1, false)), 2), Some((2, true)));
    }
}
