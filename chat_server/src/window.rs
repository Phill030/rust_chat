use crate::{config::config::ConfigManager, server::Server, types::Config};
use eframe::{egui, run_native, App, NativeOptions};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct Window {
    server: Option<Server>,
}

impl Window {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl App for Window {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.server.is_some() {
                let button = ui.button("Start server");
                if button.clicked() {
                    //
                }
            }

            if self.server.is_some() {
                ui.collapsing("Click to see what is hidden!", |ui| {
                    ui.label(format!(
                        "{:#?}",
                        &self.server.clone().unwrap().connected_clients
                    ))
                });
            }
        });
    }
}

pub fn start_window() {
    let win_option = NativeOptions::default();
    run_native(
        "Chat Server",
        win_option,
        Box::new(|cc| Box::new(Window::new(cc))),
    )
    .unwrap();
}

// https://github.com/emilk/egui/blob/master/examples/file_dialog/src/main.rs
// https://docs.rs/egui/latest/egui/
// https://doc.servo.org/egui/struct.Response.html
// https://www.youtube.com/watch?v=NtUkr_z7l84
