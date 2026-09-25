use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ClipboardItem, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Task, Window, div,
    prelude::*,
};

use crate::{
    buttons::IconButton,
    motion,
    primitives::{Icon, IconName, Tooltip},
    theme::{ActiveTheme, ControlSize, IconSize},
};

const CONFIRM: Duration = Duration::from_millis(1400);

#[derive(Default)]
struct Copied {
    count: u32,
    _reset: Option<Task<()>>,
}

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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Copied::default());
        let count = state.read(cx).count;
        let copied = state.read(cx)._reset.is_some();
        let theme = cx.theme();
        let (success, mono) = (theme.colors.success, theme.mono_family.clone());
        let text = self.text.clone();
        let button = IconButton::new("copy", IconName::Copy)
            .size(ControlSize::Sm)
            .on_click(move |_, window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
                let weak = state.downgrade();
                let reset = window.spawn(cx, async move |cx| {
                    cx.background_executor().timer(CONFIRM).await;
                    if let Err(error) = weak.update(cx, |copied, cx| {
                        copied._reset = None;
                        cx.notify();
                    }) {
                        log::error!("copyable text: reset lost its state: {error:#}");
                    }
                });
                state.update(cx, |copied, cx| {
                    copied.count += 1;
                    copied._reset = Some(reset);
                    cx.notify();
                });
            });
        let check = Icon::new(IconName::Check)
            .size(IconSize::Sm)
            .color(success)
            .with_animation(
                ("copied", count),
                Animation::new(motion::duration(motion::FAST, cx))
                    .with_easing(motion::ease_out_cubic),
                |icon, t| icon.rotate(gpui::radians((1.0 - t) * -0.6)),
            );
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_1()
            .when(self.mono, |row| row.font_family(mono))
            .child(self.text)
            .child(
                div()
                    .id("copy-slot")
                    .flex_none()
                    .size(theme.control_height(ControlSize::Sm))
                    .flex()
                    .items_center()
                    .justify_center()
                    .tooltip(Tooltip::text(if copied { "Copied" } else { "Copy" }))
                    .map(|slot| {
                        if copied {
                            slot.child(check)
                        } else {
                            slot.child(button)
                        }
                    }),
            )
    }
}
