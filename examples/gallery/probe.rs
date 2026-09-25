use std::collections::HashMap;

use ely_gpui_component::primitives::Measure;
use gpui::{App, Bounds, Global, IntoElement, ParentElement, Pixels};

/// Window bounds of named demo elements, for scripted capture.
#[derive(Default)]
pub struct Probes(HashMap<&'static str, Bounds<Pixels>>);

impl Global for Probes {}

impl Probes {
    pub fn get(key: &str, cx: &App) -> Option<Bounds<Pixels>> {
        cx.try_global::<Probes>()
            .and_then(|probes| probes.0.get(key).copied())
    }
}

/// Wraps `element` so scripts can find it by `key`.
pub fn probe(key: &'static str, element: impl IntoElement) -> Measure {
    Measure::new(key, move |bounds, _, cx| {
        cx.default_global::<Probes>().0.insert(key, bounds);
    })
    .child(element)
}
