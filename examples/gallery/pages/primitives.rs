use ely_gpui_component::{
    primitives::{Icon, IconName},
    theme::{ActiveTheme, IconSize},
};
use gpui::{AnyElement, App, InteractiveElement, IntoElement, ParentElement, Styled, Window, div};

use super::Page;
use crate::ui::{section, specimen, specimens};

pub const PAGE: Page = Page {
    number: 1,
    slug: "primitives",
    title: "Primitives",
    summary: "The smallest parts. Icons, dividers, overlays, focus.",
    render,
};

fn render(_: &mut Window, cx: &mut App) -> AnyElement {
    let colors = &cx.theme().colors;
    let sizes = [
        (IconSize::Xs, "12"),
        (IconSize::Sm, "14"),
        (IconSize::Md, "16"),
        (IconSize::Lg, "20"),
        (IconSize::Xl, "24"),
        (IconSize::Xxl, "32"),
    ];
    let tones = [
        (colors.fg, "fg"),
        (colors.fg_muted, "muted"),
        (colors.focus, "focus"),
        (colors.success, "success"),
        (colors.warning, "warning"),
        (colors.danger, "danger"),
    ];

    div()
        .child(
            section(
                "Icon",
                "Lucide, drawn as a mask. Color comes from the theme.",
                cx,
            )
            .child(specimens().children(
                sizes.map(|(size, name)| {
                    specimen(name, Icon::new(IconName::Sparkles).size(size), cx)
                }),
            ))
            .child(specimens().children(tones.map(|(color, name)| {
                specimen(name, Icon::new(IconName::CircleCheck).color(color), cx)
            }))),
        )
        .child(
            section("Icon set", "Every bundled icon.", cx).child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .children(IconName::ALL.iter().enumerate().map(|(ix, &name)| {
                        div()
                            .id(ix)
                            .size_9()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .hover(|style| style.bg(colors.hover))
                            .child(Icon::new(name).color(colors.fg_muted))
                    })),
            ),
        )
        .into_any_element()
}
