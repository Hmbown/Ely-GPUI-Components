use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    primitives::IconName,
    theme::ControlSize,
};
use gpui::{AnyElement, App, ElementId, IntoElement, ParentElement, Styled, Window, div};

use super::Page;
use crate::ui::{row, section, specimen, specimens};

pub const PAGE: Page = Page {
    number: 5,
    slug: "buttons",
    title: "Buttons & Actions",
    summary: "Seven variants, three sizes, one focus ring.",
    render,
    script: &[],
};

const VARIANTS: [(ButtonVariant, &str); 7] = [
    (ButtonVariant::Primary, "Primary"),
    (ButtonVariant::Secondary, "Secondary"),
    (ButtonVariant::Outline, "Outline"),
    (ButtonVariant::Ghost, "Ghost"),
    (ButtonVariant::Subtle, "Subtle"),
    (ButtonVariant::Danger, "Danger"),
    (ButtonVariant::Link, "Link"),
];

fn render(_: &mut Window, cx: &mut App) -> AnyElement {
    let sizes = [
        (ControlSize::Sm, "Small"),
        (ControlSize::Md, "Medium"),
        (ControlSize::Lg, "Large"),
    ];

    div()
        .child(
            section("Button", "Primary is ink. Color is saved for meaning.", cx)
                .child(row().children(VARIANTS.map(|(variant, name)| {
                    Button::new(name, name)
                        .variant(variant)
                        .on_click(move |_, _, _| log::info!("gallery: {name} clicked"))
                })))
                .child(row().children(VARIANTS.map(|(variant, name)| {
                    Button::new(ElementId::Name(format!("{name}-off").into()), name)
                        .variant(variant)
                        .disabled(true)
                }))),
        )
        .child(
            section(
                "Sizes",
                "Heights follow density. Tab moves the focus ring.",
                cx,
            )
            .child(row().children(sizes.map(|(size, name)| {
                Button::new(ElementId::Name(format!("size-{name}").into()), name)
                    .primary()
                    .size(size)
            }))),
        )
        .child(
            section(
                "With icons",
                "Leading icon for verbs. Trailing for menus.",
                cx,
            )
            .child(
                row()
                    .child(
                        Button::new("new", "New file")
                            .primary()
                            .icon(IconName::Plus),
                    )
                    .child(Button::new("share", "Share").icon(IconName::Share2))
                    .child(
                        Button::new("sort", "Sort")
                            .variant(ButtonVariant::Ghost)
                            .trailing_icon(IconName::ChevronDown),
                    )
                    .child(
                        Button::new("delete", "Delete")
                            .variant(ButtonVariant::Danger)
                            .icon(IconName::Trash2),
                    ),
            ),
        )
        .child(
            section("Icon button", "Square. Ghost by default.", cx).child(specimens().children(
                sizes.map(|(size, name)| {
                    specimen(
                        name,
                        row()
                            .gap_1()
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-a").into()),
                                    IconName::Bold,
                                )
                                .size(size),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-b").into()),
                                    IconName::Italic,
                                )
                                .size(size),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-c").into()),
                                    IconName::Underline,
                                )
                                .size(size)
                                .variant(ButtonVariant::Subtle),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-d").into()),
                                    IconName::Link,
                                )
                                .size(size)
                                .variant(ButtonVariant::Outline),
                            ),
                        cx,
                    )
                }),
            )),
        )
        .into_any_element()
}
