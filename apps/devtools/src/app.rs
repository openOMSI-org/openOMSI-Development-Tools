//! The interactive desktop application, on eframe (winit + wgpu). This path opens a window, so it
//! is never started during automated checks - those use the offscreen `render` module instead.

use crate::state::AppState;

/// The eframe application: it owns the state and draws it each frame.
pub struct DevtoolsApp {
    state: AppState,
}

impl eframe::App for DevtoolsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::draw(ui, &mut self.state);
    }
}

/// Opens the window and runs the app until it is closed.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("openOMSI Development Tools")
            .with_inner_size([1100.0, 740.0])
            .with_min_inner_size([760.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "openOMSI Development Tools",
        options,
        Box::new(|_cc| Ok(Box::new(DevtoolsApp { state: AppState::new() }))),
    )
}
