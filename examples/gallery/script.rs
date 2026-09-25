use std::{path::PathBuf, time::Duration};

use anyhow::{Context as _, Result};
use gpui::{AsyncApp, Bounds, Keystroke, Pixels, Point, WindowHandle, point, px};

use crate::{capture::snapshot, probe::Probes, shell::Gallery};

/// One scripted input, aimed at a probe by key.
pub enum Step {
    Rest,
    Hover(&'static str),
    Down(&'static str),
    Up(&'static str),
    Click(&'static str),
    /// Clicks just inside the probe's right edge.
    ClickEnd(&'static str),
    Key(&'static str),
    Wait(u64),
    Shot(&'static str),
}

const FRAME: Duration = Duration::from_millis(120);

#[derive(Clone, Copy)]
enum Mouse {
    Move,
    Down,
    Up,
}

/// Parks the pointer on empty sidebar space.
pub fn park(window: WindowHandle<Gallery>, cx: &mut AsyncApp) -> Result<()> {
    send(window, Mouse::Move, point(px(24.0), px(720.0)), cx)
}

pub async fn play(
    window: WindowHandle<Gallery>,
    steps: &[Step],
    path: impl Fn(&str) -> PathBuf,
    cx: &mut AsyncApp,
) -> Result<()> {
    for step in steps {
        match *step {
            Step::Rest => park(window, cx)?,
            Step::Hover(key) => {
                let at = target(window, key, cx).await?;
                send(window, Mouse::Move, at, cx)?;
            }
            Step::Down(key) => {
                let at = target(window, key, cx).await?;
                send(window, Mouse::Move, at, cx)?;
                send(window, Mouse::Down, at, cx)?;
            }
            Step::Up(key) => {
                let at = target(window, key, cx).await?;
                send(window, Mouse::Up, at, cx)?;
            }
            Step::Click(key) => {
                let at = target(window, key, cx).await?;
                send(window, Mouse::Move, at, cx)?;
                send(window, Mouse::Down, at, cx)?;
                send(window, Mouse::Up, at, cx)?;
            }
            Step::ClickEnd(key) => {
                let bounds = target_bounds(window, key, cx).await?;
                let at = point(bounds.right() - px(12.0), bounds.center().y);
                send(window, Mouse::Move, at, cx)?;
                send(window, Mouse::Down, at, cx)?;
                send(window, Mouse::Up, at, cx)?;
            }
            Step::Key(stroke) => {
                let stroke =
                    Keystroke::parse(stroke).with_context(|| format!("bad keystroke {stroke}"))?;
                window.update(cx, |_, window, cx| {
                    window.dispatch_keystroke(stroke, cx);
                })?;
            }
            Step::Wait(ms) => {
                cx.background_executor()
                    .timer(Duration::from_millis(ms))
                    .await
            }
            Step::Shot(name) => {
                let file = path(name);
                snapshot(&file)?;
                log::info!("script: wrote {}", file.display());
            }
        }
        cx.background_executor().timer(FRAME).await;
    }
    Ok(())
}

fn send(
    window: WindowHandle<Gallery>,
    kind: Mouse,
    at: Point<Pixels>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let height = window.update(cx, |_, window, _| window.viewport_size().height)?;
    post(kind, f64::from(at.x), f64::from(height - at.y))
}

/// Queues a real mouse event on this app. No permission needed.
#[cfg(target_os = "macos")]
fn post(kind: Mouse, x: f64, y: f64) -> Result<()> {
    use cocoa::{
        appkit::{NSApp, NSApplication, NSEvent, NSEventModifierFlags, NSEventType},
        base::{NO, id, nil},
        foundation::NSPoint,
    };
    let number = crate::capture::window_number()?;
    let kind = match kind {
        Mouse::Move => NSEventType::NSMouseMoved,
        Mouse::Down => NSEventType::NSLeftMouseDown,
        Mouse::Up => NSEventType::NSLeftMouseUp,
    };
    unsafe {
        let event = <id as NSEvent>::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure_(
            nil,
            kind,
            NSPoint::new(x, y),
            NSEventModifierFlags::empty(),
            0.0,
            i64::from(number),
            nil,
            0,
            1,
            1.0,
        );
        anyhow::ensure!(event != nil, "AppKit refused a synthetic mouse event");
        NSApp().postEvent_atStart_(event, NO);
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn post(_: Mouse, _: f64, _: f64) -> Result<()> {
    anyhow::bail!("scripted mouse input needs macOS")
}

async fn target(
    window: WindowHandle<Gallery>,
    key: &str,
    cx: &mut AsyncApp,
) -> Result<Point<Pixels>> {
    Ok(target_bounds(window, key, cx).await?.center())
}

async fn target_bounds(
    window: WindowHandle<Gallery>,
    key: &str,
    cx: &mut AsyncApp,
) -> Result<Bounds<Pixels>> {
    let bounds = cx
        .update(|cx| Probes::get(key, cx))?
        .with_context(|| format!("probe {key} never rendered"))?;
    if window.update(cx, |gallery, _, cx| gallery.reveal(bounds, cx))? {
        cx.background_executor().timer(FRAME).await;
    }
    cx.update(|cx| Probes::get(key, cx))?
        .with_context(|| format!("probe {key} vanished after scrolling"))
}
