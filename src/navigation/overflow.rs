use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div,
};

use crate::{
    forms::{Choice, Listing, listing},
    primitives::{Icon, IconName, tab_stop},
    theme::{ActiveTheme, ControlSize, IconSize, Radius},
};

type OnValue = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A button that lists every tab, for a strip too narrow to show them all.
#[derive(IntoElement)]
pub struct TabOverflowMenu {
    id: ElementId,
    tabs: Vec<Choice>,
    selected: Option<SharedString>,
    on_select: Option<OnValue>,
}

impl TabOverflowMenu {
    pub fn new(id: impl Into<ElementId>, tabs: impl IntoIterator<Item = Choice>) -> Self {
        Self {
            id: id.into(),
            tabs: tabs.into_iter().collect(),
            selected: None,
            on_select: None,
        }
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TabOverflowMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(
            !self.tabs.is_empty(),
            "tab overflow menu {:?} has no tabs",
            self.id
        );
        let focus = tab_stop((self.id.clone(), "focus").into(), true, window, cx);
        let focused = focus.is_focused(window);
        let theme = cx.theme();
        let colors = &theme.colors;
        let trigger = div()
            .id((self.id.clone(), "button"))
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(theme.control_height(ControlSize::Sm))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(if focused {
                colors.focus
            } else {
                gpui::transparent_black()
            })
            .cursor_pointer()
            .hover(|style| style.bg(colors.hover))
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(IconSize::Sm)
                    .color(colors.fg_muted),
            );
        let (id, on_select) = (self.id.clone(), self.on_select);
        let list = Listing {
            id: &self.id,
            rows: Rc::new(self.tabs),
            selected: self.selected.as_ref(),
            focused,
        };
        listing(
            list,
            trigger,
            move |value, window, cx| {
                log::info!("tab overflow menu {id:?}: {value}");
                if let Some(on_select) = &on_select {
                    on_select(value, window, cx);
                }
            },
            window,
            cx,
        )
    }
}
