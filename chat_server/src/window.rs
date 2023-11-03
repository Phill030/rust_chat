use crate::types::Config;
use eframe::{
    egui::{self, CentralPanel, TextEdit},
    run_native, App, NativeOptions,
};

#[derive(Default)]
pub struct Window {
    config: Config,
    logs: String,
}

impl Window {
    fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}

impl App for Window {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Server running on {}:{}",
                    self.config.endpoint.ip(),
                    self.config.endpoint.port()
                ));
                if ui.button("Stop server").clicked() {
                    self.logs
                        .push_str("> [ERR: config:window.rs:update] Error log message\n");
                }
            });

            ui.text_edit_multiline(&mut self.logs);
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
