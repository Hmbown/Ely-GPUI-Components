use std::{path::PathBuf, time::Duration};

use anyhow::{Context as _, Result};
use gpui::{AsyncApp, Bounds, Keystroke, Pixels, Point, WindowHandle, point, px};

use crate::{
    capture::{number, popup_number, snapshot},
    probe::{Opened, Probes},
    shell::Gallery,
};

/// One scripted input, aimed at a probe by key.
pub enum Step {
    Rest,
    Hover(&'static str),
    Down(&'static str),
    Up(&'static str),
    Click(&'static str),
    /// Clicks just inside the probe's right edge.
    ClickEnd(&'static str),
    /// Presses at an offset from the probe's top-left corner.
    DownAt(&'static str, f32, f32),
    /// Drags from the last point to an offset, in small steps.
    DragTo(&'static str, f32, f32),
    UpAt(&'static str, f32, f32),
    Key(&'static str),
    /// Types text into whatever holds focus, one key at a time.
    Type(&'static str),
    Wait(u64),
    Shot(&'static str),
    /// Photographs a window a demo opened, by its key.
    ShotWindow(&'static str, &'static str),
    CloseWindow(&'static str),
    /// Sends a keystroke to a window a demo opened.
    KeyWindow(&'static str, &'static str),
    /// Asks AppKit to close a window, as its close button would.
    CloseNative(&'static str),
    /// Fails unless the window is gone.
    ExpectClosed(&'static str),
    /// Photographs this app's frontmost popup, such as the share picker.
    ShotPopup(&'static str),
    /// Posts a key to AppKit itself; only `escape` is known.
    NativeKey(&'static str),
    /// Cancels the open file panel, whose content runs out of process.
    CancelPanel,
}

const FRAME: Duration = Duration::from_millis(120);
const DRAG_STEPS: u32 = 10;

#[derive(Clone, Copy)]
enum Mouse {
    Move,
    Down,
    Drag,
    Up,
}

/// Parks the pointer on empty sidebar space.
pub fn park(window: WindowHandle<Gallery>, cx: &mut AsyncApp) -> Result<()> {
    send(window, Mouse::Move, point(px(24.0), px(720.0)), cx)
}

fn opened(key: &str, cx: &mut AsyncApp) -> Result<gpui::AnyWindowHandle> {
    cx.update(|cx| Opened::get(key, cx))?
        .with_context(|| format!("no window opened as {key}"))
}

pub async fn play(
    window: WindowHandle<Gallery>,
    steps: &[Step],
    path: impl Fn(&str) -> PathBuf,
    cx: &mut AsyncApp,
) -> Result<()> {
    let mut last = point(px(0.0), px(0.0));
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
            Step::DownAt(key, x, y) => {
                last = target_bounds(window, key, cx).await?.origin + point(px(x), px(y));
                send(window, Mouse::Move, last, cx)?;
                send(window, Mouse::Down, last, cx)?;
            }
            Step::DragTo(key, x, y) => {
                let goal = target_bounds(window, key, cx).await?.origin + point(px(x), px(y));
                for step in 1..=DRAG_STEPS {
                    let t = step as f32 / DRAG_STEPS as f32;
                    let at = point(
                        last.x + (goal.x - last.x) * t,
                        last.y + (goal.y - last.y) * t,
                    );
                    send(window, Mouse::Drag, at, cx)?;
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;
                }
                last = goal;
            }
            Step::UpAt(key, x, y) => {
                last = target_bounds(window, key, cx).await?.origin + point(px(x), px(y));
                send(window, Mouse::Up, last, cx)?;
            }
            Step::Key(stroke) => {
                let stroke =
                    Keystroke::parse(stroke).with_context(|| format!("bad keystroke {stroke}"))?;
                press(window.into(), stroke, cx)?;
            }
            Step::Type(text) => {
                for ch in text.chars() {
                    let key = match ch {
                        ' ' => "space".to_string(),
                        ch if ch.is_ascii_uppercase() => {
                            format!("shift-{}", ch.to_ascii_lowercase())
                        }
                        ch => ch.to_string(),
                    };
                    let stroke =
                        Keystroke::parse(&key).with_context(|| format!("bad key {key}"))?;
                    press(window.into(), stroke, cx)?;
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;
                }
            }
            Step::Wait(ms) => {
                cx.background_executor()
                    .timer(Duration::from_millis(ms))
                    .await
            }
            Step::Shot(name) => {
                let file = path(name);
                snapshot(window.update(cx, |_, window, _| number(window))??, &file)?;
                log::info!("script: wrote {}", file.display());
            }
            Step::ShotWindow(key, name) => {
                let file = path(name);
                let handle = opened(key, cx)?;
                snapshot(handle.update(cx, |_, window, _| number(window))??, &file)?;
                log::info!("script: wrote {}", file.display());
            }
            Step::KeyWindow(key, stroke) => {
                let stroke =
                    Keystroke::parse(stroke).with_context(|| format!("bad keystroke {stroke}"))?;
                press(opened(key, cx)?, stroke, cx)?;
            }
            Step::CloseNative(key) => {
                let number = opened(key, cx)?.update(cx, |_, window, _| number(window))??;
                perform_close(number)?;
            }
            Step::ShotPopup(name) => {
                let file = path(name);
                snapshot(popup_number()?, &file)?;
                log::info!("script: wrote {}", file.display());
            }
            Step::NativeKey(key) => {
                anyhow::ensure!(key == "escape", "native key {key} is not known");
                let number = window.update(cx, |_, window, _| number(window))??;
                post_escape(number)?;
            }
            Step::CancelPanel => cancel_panel()?,
            Step::ExpectClosed(key) => {
                let handle = opened(key, cx)?;
                let open = cx.update(|cx| cx.windows().contains(&handle))?;
                anyhow::ensure!(!open, "window {key} is still open");
                cx.update(|cx| Opened::take(key, cx))?;
                log::info!("script: {key} closed");
            }
            Step::CloseWindow(key) => {
                let handle = opened(key, cx)?;
                handle.update(cx, |_, window, _| window.remove_window())?;
                cx.update(|cx| Opened::take(key, cx))?;
            }
        }
        cx.background_executor().timer(FRAME).await;
    }
    Ok(())
}

/// Types a keystroke without holding the root view, so the window can redraw.
fn press(window: gpui::AnyWindowHandle, stroke: Keystroke, cx: &mut AsyncApp) -> Result<()> {
    window.update(cx, |_, window, cx| {
        window.dispatch_keystroke(stroke, cx);
    })
}

fn send(
    window: WindowHandle<Gallery>,
    kind: Mouse,
    at: Point<Pixels>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let (height, number) = window.update(cx, |_, window, _| {
        (window.viewport_size().height, number(window))
    })?;
    post(kind, f64::from(at.x), f64::from(height - at.y), number?)
}

/// Queues a real mouse event on this app. No permission needed.
#[cfg(target_os = "macos")]
fn post(kind: Mouse, x: f64, y: f64, number: u32) -> Result<()> {
    use cocoa::{
        appkit::{NSApp, NSApplication, NSEvent, NSEventModifierFlags, NSEventType},
        base::{NO, id, nil},
        foundation::NSPoint,
    };
    let kind = match kind {
        Mouse::Move => NSEventType::NSMouseMoved,
        Mouse::Down => NSEventType::NSLeftMouseDown,
        Mouse::Drag => NSEventType::NSLeftMouseDragged,
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

/// Queues an Escape key press on this app, for AppKit popovers.
#[cfg(target_os = "macos")]
fn post_escape(number: u32) -> Result<()> {
    use cocoa::{
        appkit::{NSApp, NSApplication, NSEvent, NSEventModifierFlags, NSEventType},
        base::{NO, id, nil},
        foundation::{NSPoint, NSString},
    };
    unsafe {
        let escape = NSString::alloc(nil).init_str("\u{1b}");
        let event = <id as NSEvent>::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode_(
            nil,
            NSEventType::NSKeyDown,
            NSPoint::new(0.0, 0.0),
            NSEventModifierFlags::empty(),
            0.0,
            i64::from(number),
            nil,
            escape,
            escape,
            NO,
            53,
        );
        anyhow::ensure!(event != nil, "AppKit refused a synthetic key event");
        NSApp().postEvent_atStart_(event, NO);
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn post_escape(_: u32) -> Result<()> {
    anyhow::bail!("native keys need macOS")
}

#[cfg(target_os = "macos")]
fn cancel_panel() -> Result<()> {
    use cocoa::{
        appkit::NSApp,
        base::{BOOL, NO, id, nil},
    };
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let key: id = msg_send![NSApp(), keyWindow];
        anyhow::ensure!(key != nil, "no key window to cancel");
        let panel: BOOL = msg_send![key, isKindOfClass: class!(NSSavePanel)];
        anyhow::ensure!(panel != NO, "the key window is not a file panel");
        let _: () = msg_send![key, cancel: nil];
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn cancel_panel() -> Result<()> {
    anyhow::bail!("file panels need macOS")
}

/// Runs `performClose:` on a window, which asks before closing.
#[cfg(target_os = "macos")]
fn perform_close(number: u32) -> Result<()> {
    use cocoa::{
        appkit::NSApp,
        base::{id, nil},
    };
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let window: id = msg_send![NSApp(), windowWithWindowNumber: i64::from(number)];
        anyhow::ensure!(window != nil, "no AppKit window numbered {number}");
        let _: () = msg_send![window, performClose: nil];
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn perform_close(_: u32) -> Result<()> {
    anyhow::bail!("native close needs macOS")
}

#[cfg(not(target_os = "macos"))]
fn post(_: Mouse, _: f64, _: f64, _: u32) -> Result<()> {
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
