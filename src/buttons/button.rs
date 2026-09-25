use gpui::{
    App, ClickEvent, ElementId, FontWeight, Hsla, IntoElement, MouseButton, RenderOnce,
    SharedString, Window, div, prelude::*, transparent_black,
};

use crate::{
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, Mix, Palette, Radius, TextSize},
};

pub(crate) type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    #[default]
    Secondary,
    Outline,
    Ghost,
    Subtle,
    Danger,
    Link,
}

pub(crate) struct Tone {
    pub bg: Hsla,
    pub hover: Hsla,
    pub pressed: Hsla,
    pub fg: Hsla,
    pub border: Option<Hsla>,
}

pub(crate) fn tone(variant: ButtonVariant, colors: &Palette) -> Tone {
    let clear = transparent_black();
    match variant {
        ButtonVariant::Primary => Tone {
            bg: colors.accent,
            hover: colors.accent_hover,
            pressed: colors.accent_hover.mix(&colors.on_accent, 0.12),
            fg: colors.on_accent,
            border: None,
        },
        ButtonVariant::Secondary => Tone {
            bg: colors.surface,
            hover: colors.hover,
            pressed: colors.active,
            fg: colors.fg,
            border: Some(colors.border),
        },
        ButtonVariant::Outline => Tone {
            bg: clear,
            hover: colors.hover,
            pressed: colors.active,
            fg: colors.fg,
            border: Some(colors.border_strong),
        },
        ButtonVariant::Ghost => Tone {
            bg: clear,
            hover: colors.hover,
            pressed: colors.active,
            fg: colors.fg,
            border: None,
        },
        ButtonVariant::Subtle => Tone {
            bg: colors.hover,
            hover: colors.active,
            pressed: colors.active.mix(&colors.border_strong, 0.5),
            fg: colors.fg,
            border: None,
        },
        ButtonVariant::Danger => Tone {
            bg: colors.danger,
            hover: colors.danger.mix(&colors.fg, 0.14),
            pressed: colors.danger.mix(&colors.fg, 0.24),
            fg: colors.on_accent,
            border: None,
        },
        ButtonVariant::Link => Tone {
            bg: clear,
            hover: clear,
            pressed: clear,
            fg: colors.link,
            border: None,
        },
    }
}

pub(crate) fn label_size(size: ControlSize) -> (TextSize, IconSize) {
    match size {
        ControlSize::Sm => (TextSize::Sm, IconSize::Xs),
        ControlSize::Md => (TextSize::Base, IconSize::Sm),
        ControlSize::Lg => (TextSize::Md, IconSize::Md),
    }
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: bool,
    full_width: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            trailing_icon: None,
            variant: ButtonVariant::default(),
            size: ControlSize::default(),
            disabled: false,
            full_width: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let tone = tone(self.variant, &theme.colors);
        let (text, icon_size) = label_size(self.size);
        let link = self.variant == ButtonVariant::Link;
        let icon = |name| Icon::new(name).size(icon_size).color(tone.fg);

        div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap_1p5()
            .h(theme.control_height(self.size))
            .when(!link, |el| el.px(theme.control_padding(self.size)))
            .rounded(theme.radius(Radius::Md))
            .text_size(theme.text_size(text))
            .font_weight(FontWeight::MEDIUM)
            .text_color(tone.fg)
            .bg(tone.bg)
            .when_some(tone.border, |el, border| el.border_1().border_color(border))
            .when(self.full_width, |el| el.w_full())
            .when_some(self.icon, |el, name| el.child(icon(name)))
            .child(self.label)
            .when_some(self.trailing_icon, |el, name| el.child(icon(name)))
            .map(|el| {
                if self.disabled {
                    return el.opacity(0.45).cursor_not_allowed();
                }
                el.cursor_pointer()
                    .tab_index(0)
                    .hover(|style| {
                        let style = style.bg(tone.hover);
                        if link { style.underline() } else { style }
                    })
                    .active(|style| style.bg(tone.pressed))
                    .focus(|style| style.shadow(theme.focus_ring()))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .when_some(self.on_click, |el, handler| el.on_click(handler))
            })
    }
}
