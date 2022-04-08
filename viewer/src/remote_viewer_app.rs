use crate::App;
use eframe::egui;

/// Connects to a server over `WebSockets`.
#[derive(Default)]
pub struct RemoteViewerApp {
    url: String,
    app: Option<(comms::Connection, App)>,
}

impl RemoteViewerApp {
    /// url to rerun server
    pub fn new(
        egui_ctx: egui::Context,
        storage: Option<&dyn eframe::Storage>,
        url: String,
    ) -> Self {
        let mut slf = Self { url, app: None };
        slf.connect(egui_ctx, storage);
        slf
    }

    fn connect(&mut self, egui_ctx: egui::Context, storage: Option<&dyn eframe::Storage>) {
        let (tx, rx) = std::sync::mpsc::channel();

        let connection = comms::Connection::viewer_to_server(
            self.url.clone(),
            move |log_msg: log_types::LogMsg| {
                if tx.send(log_msg).is_ok() {
                    egui_ctx.request_repaint(); // Wake up UI thread
                    std::ops::ControlFlow::Continue(())
                } else {
                    tracing::info!("Failed to send log message to viewer - closing");
                    std::ops::ControlFlow::Break(())
                }
            },
        )
        .unwrap(); // TODO: handle error

        let app = crate::App::new(storage, rx);

        self.app = Some((connection, app));
    }
}

impl eframe::App for RemoteViewerApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some((_, app)) = &mut self.app {
            app.save(storage);
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("server").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                if ui.text_edit_singleline(&mut self.url).lost_focus()
                    && ui.input().key_pressed(egui::Key::Enter)
                {
                    if let Some(storage) = frame.storage_mut() {
                        if let Some((_, mut app)) = self.app.take() {
                            app.save(storage);
                        }
                    }
                    self.connect(ctx.clone(), frame.storage());
                }
            });
        });

        if let Some((_, app)) = &mut self.app {
            app.update(ctx, frame);
        }
    }
}
