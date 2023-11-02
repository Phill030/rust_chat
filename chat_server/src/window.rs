use crate::types::Config;
use eframe::{egui, run_native, App, NativeOptions};

pub struct Window {
    config: Config,
}

impl Window {
    fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self { config }
    }
}

impl App for Window {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "Server running on {}:{}",
                self.config.endpoint.ip(),
                self.config.endpoint.port()
            ));
        });
    }
}

pub fn start_window(config: Config) {
    let win_option = NativeOptions::default();
    run_native(
        "Chat Server",
        win_option,
        Box::new(move |cc| {
            let config_cloned = config.clone();

            Box::new(Window::new(cc, config_cloned))
        }),
    )
    .unwrap();
}

// https://github.com/emilk/egui/blob/master/examples/file_dialog/src/main.rs
// https://docs.rs/egui/latest/egui/
// https://doc.servo.org/egui/struct.Response.html
// https://www.youtube.com/watch?v=NtUkr_z7l84
