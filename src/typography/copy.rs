use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*,
};

use crate::{buttons::CopyButton, theme::ActiveTheme};

/// Text with a copy button that confirms with a check.
#[derive(IntoElement)]
pub struct CopyableText {
    id: ElementId,
    text: SharedString,
    mono: bool,
}

impl CopyableText {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            mono: false,
        }
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
}

impl RenderOnce for CopyableText {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let mono = cx.theme().mono_family.clone();
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_1()
            .when(self.mono, |row| row.font_family(mono))
            .child(self.text.clone())
            .child(CopyButton::new("copy-slot", self.text))
    }
}
