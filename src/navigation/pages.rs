use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    primitives::IconName,
    theme::{ActiveTheme, ControlSize},
};

/// The first, the last, `current` and `near` pages either side; `None` marks a gap of two or more.
pub(crate) fn page_list(current: usize, total: usize, near: usize) -> Vec<Option<usize>> {
    assert!(
        total >= 1 && (1..=total).contains(&current),
        "page {current} is outside 1 to {total}"
    );
    if total == 1 {
        return vec![Some(1)];
    }
    let start = current.saturating_sub(near).max(2);
    let end = (current + near).min(total - 1);
    let mut pages = vec![Some(1)];
    if start > 3 {
        pages.push(None);
    } else {
        pages.extend((2..start).map(Some));
    }
    pages.extend((start..=end).map(Some));
    if end + 2 < total {
        pages.push(None);
    } else {
        pages.extend((end + 1..total).map(Some));
    }
    pages.push(Some(total));
    pages
}

type OnPage = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Numbered pages between previous and next; long runs fold into gaps.
#[derive(IntoElement)]
pub struct Pagination {
    id: ElementId,
    page: usize,
    pages: usize,
    near: usize,
    on_change: Option<OnPage>,
}

impl Pagination {
    /// `page` counts from 1.
    pub fn new(id: impl Into<ElementId>, page: usize, pages: usize) -> Self {
        Self {
            id: id.into(),
            page,
            pages,
            near: 1,
            on_change: None,
        }
    }

    /// Pages shown on each side of the current one.
    pub fn near(mut self, near: usize) -> Self {
        self.near = near;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let list = page_list(self.page, self.pages, self.near);
        let (page, pages) = (self.page, self.pages);
        let go = {
            let (id, on_change) = (self.id.clone(), self.on_change);
            Rc::new(move |to: usize, window: &mut Window, cx: &mut App| {
                log::info!("pagination {id:?}: page {to}");
                if let Some(on_change) = &on_change {
                    on_change(to, window, cx);
                }
            })
        };
        let subtle = cx.theme().colors.fg_subtle;
        let (back, ahead) = (go.clone(), go.clone());
        let cells = list
            .into_iter()
            .enumerate()
            .map(move |(ix, entry)| match entry {
                Some(number) => {
                    let go = go.clone();
                    Button::new(("page", number), number.to_string())
                        .size(ControlSize::Sm)
                        .variant(if number == page {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .on_click(move |_, window, cx| {
                            if number != page {
                                go(number, window, cx);
                            }
                        })
                        .into_any_element()
                }
                None => div()
                    .id(("gap", ix))
                    .px_1()
                    .text_color(subtle)
                    .child("…")
                    .into_any_element(),
            });
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_1()
            .child(
                IconButton::new("previous", IconName::ChevronLeft)
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .tooltip("Previous page")
                    .disabled(page == 1)
                    .on_click(move |_, window, cx| back(page - 1, window, cx)),
            )
            .children(cells)
            .child(
                IconButton::new("next", IconName::ChevronRight)
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .tooltip("Next page")
                    .disabled(page == pages)
                    .on_click(move |_, window, cx| ahead(page + 1, window, cx)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::page_list;

    fn shown(current: usize, total: usize) -> String {
        page_list(current, total, 1)
            .into_iter()
            .map(|page| page.map_or("…".to_string(), |page| page.to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn gaps_fold_long_runs_and_never_hide_one_page() {
        assert_eq!(shown(6, 20), "1 … 5 6 7 … 20");
        assert_eq!(shown(1, 20), "1 2 … 20");
        assert_eq!(shown(4, 20), "1 2 3 4 5 … 20");
        assert_eq!(shown(17, 20), "1 … 16 17 18 19 20");
        assert_eq!(shown(20, 20), "1 … 19 20");
        assert_eq!(shown(1, 1), "1");
        assert_eq!(shown(2, 2), "1 2");
        assert_eq!(shown(3, 5), "1 2 3 4 5");
    }
}
