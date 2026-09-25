mod capture;
mod pages;
mod probe;
mod script;
mod shell;
mod ui;

use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::Assets;
use gpui::{
    App, AppContext, Application, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions,
    actions, point, px, size,
};

actions!(gallery, [Quit]);

struct Args {
    page: Option<usize>,
    capture: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        page: None,
        capture: None,
    };
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .with_context(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--page" => {
                args.page = Some(pages::find(&value).with_context(|| format!("no page {value}"))?)
            }
            "--capture" => args.capture = Some(PathBuf::from(value)),
            other => bail!("unknown flag {other}"),
        }
    }
    Ok(args)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = parse_args()?;
    Application::new()
        .with_assets(Assets)
        .run(move |cx: &mut App| {
            ely_gpui_component::init(cx);
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Ely".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(16.0), px(18.0))),
                }),
                window_min_size: Some(size(px(880.0), px(560.0))),
                ..Default::default()
            };
            let window = cx
                .open_window(options, |window, cx| {
                    cx.new(|cx| shell::Gallery::new(args.page.unwrap_or(0), window, cx))
                })
                .expect("gallery window failed to open");
            cx.activate(true);
            if let Some(dir) = args.capture {
                capture::run(window, dir, args.page, cx);
            }
        });
    Ok(())
}
