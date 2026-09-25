use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    forms::{Input, Switch, TextInput},
    overlays::{AlertDialog, ConfirmDialog, Dialog, HoverCard, Popover, PromptDialog},
    primitives::{IconName, Severity},
    theme::{ActiveTheme, TextSize},
    typography::Caption,
};
use gpui::{
    AnyElement, App, Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::*,
};

use super::Page;
use crate::{
    probe::probe,
    script::Step,
    ui::{keep, row, section, set, specimen},
};

pub const PAGE: Page = Page {
    number: 9,
    slug: "overlays",
    title: "Overlays",
    summary: "Panels over the page: anchored to a control, over a scrim, or over everything.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("share"),
    Step::Wait(400),
    Step::Shot("popover"),
    Step::Key("escape"),
    Step::Hover("ada"),
    Step::Wait(900),
    Step::Shot("hovercard"),
    Step::Hover("share"),
    Step::Wait(400),
    Step::Click("edit-profile"),
    Step::Wait(500),
    Step::Shot("dialog"),
    Step::Key("escape"),
    Step::Click("alert"),
    Step::Wait(500),
    Step::Shot("alert"),
    Step::Key("escape"),
    Step::Click("confirm"),
    Step::Wait(500),
    Step::Shot("confirm"),
    Step::Key("escape"),
    Step::Click("prompt"),
    Step::Wait(400),
    Step::Type("drafts/2027"),
    Step::Wait(300),
    Step::Shot("prompt"),
    Step::Key("escape"),
    Step::Click("fullscreen"),
    Step::Wait(600),
    Step::Shot("fullscreen"),
    Step::Key("escape"),
    Step::Rest,
];

fn popovers(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let public = keep("share-public", || true, window, cx);
    let on = *public.read(cx);
    let theme = cx.theme();
    let muted = theme.colors.fg_muted;
    let panel = move |_: &mut Window, cx: &mut App| {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .w_72()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Share this page"),
            )
            .child(
                div()
                    .text_color(theme.colors.fg_muted)
                    .child("Anyone with the link can view it."),
            )
            .child(
                Switch::new("share-switch", on)
                    .label("Public link")
                    .on_change(move |next, _, cx| set(&public, next, cx)),
            )
            .child(
                Button::new("share-copy", "Copy link")
                    .icon(IconName::Link)
                    .full_width(),
            )
    };
    section(
        "Popover / DropdownPanel / InlinePopup",
        "A panel of any content under its button. A press outside or Escape closes it, and Tab stays inside. The Link variant sets the trigger inline.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "share",
                Popover::new("share", "Share", panel).icon(IconName::Share2),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_color(muted)
                    .child("Read the")
                    .child(
                        Popover::new("terms", "terms", |_, cx| {
                            div()
                                .w_72()
                                .text_color(cx.theme().colors.fg_muted)
                                .child("Plain words: you own your work, and we keep it private.")
                        })
                        .variant(ButtonVariant::Link),
                    )
                    .child("before you share."),
            ),
    )
}

fn profile(cx: &App) -> AnyElement {
    let theme = cx.theme();
    let colors = &theme.colors;
    div()
        .flex()
        .flex_col()
        .gap_3()
        .w_64()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_10()
                        .rounded_full()
                        .bg(colors.hover)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("AL"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Ada Lovelace"),
                        )
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .text_color(colors.fg_subtle)
                                .child("Design · London"),
                        ),
                ),
        )
        .child(
            div()
                .text_color(colors.fg_muted)
                .child("Writes the first programs for machines not yet built."),
        )
        .child(Button::new("ada-follow", "Follow").primary())
        .into_any_element()
}

fn hover_card(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    section(
        "HoverCard",
        "Rest the pointer on a name and a card opens. It stays while the pointer is on either, and never takes focus.",
        cx,
    )
    .child(specimen(
        "rest the pointer on the name",
        probe(
            "ada",
            HoverCard::new(
                "ada",
                div()
                    .text_color(theme.colors.link)
                    .font_weight(FontWeight::MEDIUM)
                    .child("@ada"),
                |_, cx| profile(cx),
            ),
        ),
        cx,
    ))
}

/// Which dialog is open, if any.
#[derive(Clone, Copy, PartialEq)]
enum Open {
    Profile,
    Alert,
    Confirm,
    Prompt,
    Fullscreen,
}

fn opener(
    state: &Entity<Option<Open>>,
    id: &'static str,
    label: &'static str,
    open: Open,
) -> impl IntoElement {
    let state = state.clone();
    probe(
        id,
        Button::new(id, label).on_click(move |_, _, cx| set(&state, Some(open), cx)),
    )
}

fn dialogs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("overlay-open", || None::<Open>, window, cx);
    let said = keep("overlay-said", || None::<SharedString>, window, cx);
    let (name, email, rename) = {
        let make = |window: &mut Window, cx: &mut App, text: &str| {
            let text = text.to_string();
            cx.new(|cx| {
                let mut input = TextInput::new(window, cx);
                input.set_text(text, cx);
                input
            })
        };
        let fields = window.use_keyed_state("overlay-fields", cx, |window, cx| {
            (
                make(window, cx, "Ada Lovelace"),
                make(window, cx, "ada@ely.dev"),
                make(window, cx, "Roadmap"),
            )
        });
        fields.read(cx).clone()
    };
    let shut = {
        let open = open.clone();
        move |_: &mut Window, cx: &mut App| set(&open, None, cx)
    };
    let caption = said
        .read(cx)
        .clone()
        .map_or("each opens over a scrim; Escape closes it".into(), |said| {
            said.to_string()
        });
    let overlay: Option<AnyElement> = (*open.read(cx)).map(|which| {
        let (saved, confirmed, renamed) = (said.clone(), said.clone(), said.clone());
        match which {
            Open::Profile => Dialog::new("profile", "Edit profile", shut.clone())
                .detail("How others see you across the workspace.")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(Input::new(&name))
                        .child(Input::new(&email)),
                )
                .action(|close| {
                    Button::new("profile-cancel", "Cancel")
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |_, window, cx| close(window, cx))
                })
                .action(move |close| {
                    Button::new("profile-save", "Save")
                        .primary()
                        .on_click(move |_, window, cx| {
                            set(&saved, Some("profile saved".into()), cx);
                            close(window, cx);
                        })
                })
                .into_any_element(),
            Open::Alert => AlertDialog::new(
                "expired",
                Severity::Warning,
                "Session expired",
                "Sign in again to keep your changes. Nothing you wrote is lost.",
                shut.clone(),
            )
            .label("Sign in")
            .into_any_element(),
            Open::Confirm => ConfirmDialog::new(
                "delete",
                "Delete this project?",
                "Its pages, files and history go with it. This cannot be undone.",
                shut.clone(),
            )
            .confirm("Delete project")
            .destructive()
            .on_confirm(move |_, cx| set(&confirmed, Some("project deleted".into()), cx))
            .into_any_element(),
            Open::Prompt => PromptDialog::new("rename", "Rename", &rename, shut.clone())
                .label("New name")
                .submit("Rename")
                .check(|text| match text.trim() {
                    "" => Err("A name cannot be empty.".into()),
                    text if text.contains('/') => Err("Names cannot hold a slash.".into()),
                    _ => Ok(()),
                })
                .on_submit(move |text, _, cx| {
                    set(&renamed, Some(format!("renamed to {text}").into()), cx)
                })
                .into_any_element(),
            Open::Fullscreen => Dialog::new("notes", "Release notes", shut.clone())
                .detail("Version 0.9 · September 2026")
                .fullscreen()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .max_w(
                            cx.theme()
                                .container_width(ely_gpui_component::theme::ContainerSize::Sm),
                        )
                        .child(Caption::new("Menus nest, filter and follow their buttons."))
                        .child(Caption::new("Palettes pick on release, as buttons do."))
                        .child(Caption::new("Overlays hand focus back on every close.")),
                )
                .into_any_element(),
        }
    });
    section(
        "Dialog / Modal / AlertDialog / ConfirmDialog / PromptDialog / FullscreenDialog",
        "A card over a scrim; Tab stays inside and focus returns on close. An alert holds until acknowledged, a confirm asks first, a prompt checks its one line, and a fullscreen dialog fills the window.",
        cx,
    )
    .child(specimen(
        caption,
        row()
            .child(opener(&open, "edit-profile", "Edit profile", Open::Profile))
            .child(opener(&open, "alert", "Session expired", Open::Alert))
            .child(opener(&open, "confirm", "Delete project", Open::Confirm))
            .child({
                let (open, rename) = (open.clone(), rename.clone());
                probe(
                    "prompt",
                    Button::new("prompt", "Rename").on_click(move |_, _, cx| {
                        rename.update(cx, |input, cx| input.set_text("Roadmap", cx));
                        set(&open, Some(Open::Prompt), cx);
                    }),
                )
            })
            .child(opener(&open, "fullscreen", "Release notes", Open::Fullscreen)),
        cx,
    ))
    .children(overlay)
}

fn pointers(cx: &App) -> impl IntoElement + use<> {
    section(
        "Tooltip / Sheet / Drawer",
        "They live with the pieces they are made of.",
        cx,
    )
    .child(Caption::new(
        "Tooltip is primitives::Tooltip, on Primitives, chapter 1. Sheet and Drawer are layout::Sheet and layout::Drawer, on Layout, chapter 3.",
    ))
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(popovers(window, cx))
        .child(hover_card(cx))
        .child(dialogs(window, cx))
        .child(pointers(cx))
        .into_any_element()
}
