use eframe::Result;
use qsolog::app::QsoApp;
use std::path::PathBuf;

fn main() -> Result<()> {
    let db_path = PathBuf::from("qso.db");

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "QSOLink",
        native_options,
        Box::new(|cc| Ok(Box::new(QsoApp::new(cc, Some(db_path))))),
    )
}
