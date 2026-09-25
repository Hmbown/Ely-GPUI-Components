use gpui::{
    App, ElementId, Entity, IntoElement, PathPromptOptions, RenderOnce, SharedString, Window,
};

use super::{Input, TextInput};
use crate::{
    buttons::{Button, ButtonVariant},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize},
};

/// A path field with Browse, which opens the system's file dialog.
#[derive(IntoElement)]
pub struct PathInput {
    id: ElementId,
    state: Entity<TextInput>,
    directories: bool,
}

impl PathInput {
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextInput>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            directories: false,
        }
    }

    /// Chooses folders instead of files.
    pub fn directories(mut self) -> Self {
        self.directories = true;
        self
    }
}

impl RenderOnce for PathInput {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let subtle = cx.theme().colors.fg_subtle;
        let (state, directories) = (self.state.clone(), self.directories);
        let icon = if directories {
            IconName::Folder
        } else {
            IconName::File
        };
        Input::new(&self.state)
            .prefix(Icon::new(icon).size(IconSize::Sm).color(subtle))
            .suffix(
                Button::new(self.id, "Browse…")
                    .size(ControlSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, window, cx| {
                        let chosen = cx.prompt_for_paths(PathPromptOptions {
                            files: !directories,
                            directories,
                            multiple: false,
                            prompt: Some(SharedString::from("Choose")),
                        });
                        log::info!("path input: dialog opened");
                        let state = state.clone();
                        window
                            .spawn(cx, async move |cx| {
                                let path = match chosen.await {
                                    Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                    Ok(Ok(None)) => {
                                        log::info!("path input: dialog cancelled");
                                        return;
                                    }
                                    Ok(Err(error)) => {
                                        log::error!("path input: dialog failed: {error:#}");
                                        return;
                                    }
                                    Err(_) => {
                                        log::error!("path input: dialog closed without an answer");
                                        return;
                                    }
                                };
                                let Some(path) = path else {
                                    log::error!("path input: dialog chose nothing");
                                    return;
                                };
                                let shown = path.display().to_string();
                                log::info!("path input: chose a path");
                                if let Err(error) =
                                    state.update(cx, |input, cx| input.set_text(shown, cx))
                                {
                                    log::error!("path input: field gone: {error:#}");
                                }
                            })
                            .detach();
                    }),
            )
    }
}
