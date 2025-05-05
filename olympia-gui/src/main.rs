use eframe::egui;
use egui::Ui;
use egui_file_dialog::FileDialog;
use egui_graphs::{DefaultGraphView, Graph as EguiGraph, GraphView, SettingsNavigation};
use olympia_core::Program;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    );
}

#[derive(Default)]
struct MyEguiApp {
    file_dialog: FilePickup,
    app_state: Arc<RwLock<AppState>>,
    settings_nav: SettingsNavigation,
    graph: Option<EguiGraph>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let app_state = Arc::new(RwLock::new(AppState::default()));
        let file_dialog = FilePickup::new(Arc::clone(&app_state));

        Self {
            file_dialog,
            app_state,
            graph: Option::default(),
            settings_nav: SettingsNavigation::new().with_zoom_and_pan_enabled(true),
        }
    }
}

#[derive(Default)]
struct AppState {
    program: Option<Program>,
    picked_file: Option<PathBuf>,
}

#[derive(Default)]
struct FilePickup {
    file_dialog: FileDialog,
    app_state: Arc<RwLock<AppState>>,
}

impl FilePickup {
    pub fn new(app_state: Arc<RwLock<AppState>>) -> Self {
        Self {
            file_dialog: FileDialog::default(),
            app_state,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        if ui.button("Pick file").clicked() {
            // Open the file dialog to pick a file.
            self.file_dialog.pick_file();
        }
        // Update the dialog
        self.file_dialog.update(ctx);

        {
            let mut app_state_write = self.app_state.write().unwrap();

            if let Some(path) = self.file_dialog.take_picked() {
                app_state_write.picked_file = Some(path);
            }
        }

        {
            let mut app_state_write = self.app_state.write().unwrap();

            if let Some(ref path) = app_state_write.picked_file {
                match olympia_core::load(&path) {
                    Ok(program) => app_state_write.program = Some(program),
                    Err(msg) => {
                        ui.label(format!("Error: {}", msg));
                    }
                }
            }
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Update the dialog
            self.file_dialog.update(ctx, ui);

            if let None = self.graph {
                let app_state = self.app_state.read().unwrap();

                if let Some(ref program) = app_state.program {
                    let g = program.call_graph().map(|_, _| {}, |_, _| {});
                    ui.label(format!("Node Count : {}", g.node_count()));
                    self.graph = Some(EguiGraph::<(), ()>::from(&g));
                }
            } else if let Some(graph) = &mut self.graph {
                let mut view = DefaultGraphView::new(graph).with_navigations(&self.settings_nav);
                ui.add(&mut view);
            }
        });
    }
}
