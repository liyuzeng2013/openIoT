use eframe::egui;
use shared::types::*;

mod api;
mod views;

use views::{login_view, dashboard_view};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("OpenIoT 客户端"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenIoT",
        options,
        Box::new(|_cc| Ok(Box::new(IoTApp::default()))),
    )
}

struct IoTApp {
    token: Option<String>,
    username: Option<String>,
    devices: Vec<Device>,
    current_view: AppView,
    login_username: String,
    login_password: String,
    status_message: Option<String>,
}

enum AppView {
    Login,
    Dashboard,
}

impl Default for IoTApp {
    fn default() -> Self {
        Self {
            token: None,
            username: None,
            devices: Vec::new(),
            current_view: AppView::Login,
            login_username: String::new(),
            login_password: String::new(),
            status_message: None,
        }
    }
}

impl eframe::App for IoTApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_view {
            AppView::Login => {
                login_view::show(self, ctx);
            }
            AppView::Dashboard => {
                dashboard_view::show(self, ctx);
            }
        }
    }
}
