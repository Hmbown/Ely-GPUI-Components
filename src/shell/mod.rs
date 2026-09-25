mod activity;
mod bars;
mod chrome;
mod crash;
mod dialogs;
mod frame;
mod native;
mod status;
mod switcher;
mod tabs;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod update;
mod windows;

pub use activity::{ActivityBar, ActivityItem};
pub use bars::{StatusBar, StatusBarItem, Toolbar, ToolbarGroup, ToolbarSeparator};
pub use chrome::{ControlsStyle, TitleBar, WindowControls, drag_region};
pub use crash::CrashReporter;
pub use dialogs::{AboutDialog, SplashScreen};
pub use frame::ResizeBorder;
pub use native::{TrayIcon, TrayItem, dock_badge, set_dock_badge};
pub use status::{Connectivity, OfflineIndicator, ZoomControl};
pub use switcher::WindowSwitcher;
pub use tabs::{TabBar, WindowTab};
pub use update::{UpdateBanner, UpdateDialog, UpdateState};
pub use windows::{Corner, Hosted, MiniWindow, WindowManager, open_hosted};
