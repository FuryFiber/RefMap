mod core;
mod app;
use crate::app::MindMapApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "RefMap",
        options,
        Box::new(|_cc| Ok(Box::new(MindMapApp::default()))),
    )
}

