use std::sync::Arc;

use eframe::{egui, run_native, App, NativeOptions};

use crate::{config::config::ConfigManager, server::Server};

#[derive(Default, Clone)]
pub struct Window {
    server: Arc<Option<Server>>,
}

impl Window {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl App for Window {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.server.is_some() {
                let button = ui.button("Start server");
                if button.clicked() {
                    let mut cloned_server = self.clone().server;
                    tokio::spawn(async move {
                        let config = ConfigManager::initialize_or_create().await.unwrap();
                        cloned_server = Arc::new(Some(Server::create(config.endpoint).unwrap()));
                    });

                    self.server = cloned_server;
                }
            }

            if self.server.as_ref().is_some() {
                ui.collapsing("Click to see what is hidden!", |ui| {
                    ui.label(format!(
                        "{:#?}",
                        &self.server.as_ref().clone().unwrap().connected_clients
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
