use std::fs;
use std::path::PathBuf;

use egui_extras::TableBody;

use crate::Uri;
use crate::{printer, printer_settings::PrinterSettings};

pub(crate) struct UiImage {
    pub path: PathBuf,
    print_success: usize,
    print_error: usize,
}

impl UiImage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            print_success: 0,
            print_error: 0,
        }
    }
    pub fn add_to_table_body(&mut self, body: &mut TableBody, callback: impl FnOnce(&mut UiImage)) {
        body.row(80.0, |mut row| {
            row.col(|ui| {
                let uri = format!("file://{}", self.path.display().to_string());
                ui.image(uri);
            });
            row.col(|ui| {
                ui.label(self.path.display().to_string());
            });
            match self.is_valid() {
                true => {
                    row.col(|ui| {
                        ui.label("OK");
                    });
                    row.col(|ui| {
                        if ui.button("Edit").clicked() {
                            let _ = opener::open(self.path.clone());
                        }
                        if ui.button("Folder").clicked() {
                            let _ = opener::open(self.path.clone().parent().unwrap());
                        }

                        if ui.button("Print").clicked() {
                            callback(self);
                        }
                    });
                }
                false => {
                    row.col(|ui| {
                        ui.label("File not found");
                    });
                    row.col(|ui| {
                        ui.add_enabled(false, egui::Button::new("Print"));
                    });
                }
            };
        });
    }
    pub fn is_valid(&self) -> bool {
        fs::exists(self.path.clone()).is_ok_and(|f| f == true)
    }

    pub(crate) fn print(&mut self, host: &Uri, printer_settings: &PrinterSettings) {
        match printer::print_file(
            host,
            &printer_settings.printer_name,
            &printer_settings.media_size,
            &printer_settings.media_type,
            &self.path,
        ) {
            Ok(_) => self.print_success += 1,
            Err(_) => self.print_error += 1,
        }
    }
}
