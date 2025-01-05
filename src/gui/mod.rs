use std::{collections::HashMap, path::PathBuf};

mod ui_image;
mod message_box;

use crate::{
    printer::*,
    printer_settings::{self, PrinterSettings},
    Uri,
};

use egui_extras::{Column, TableBuilder};
use log::{error, info};

use eframe::egui;
use ui_image::UiImage;

use crate::printer;

struct PrintGui {
    printer_settings: PrinterSettings,
    printers: HashMap<String, Printer>,
    images: Vec<UiImage>,
    host: Uri,
}

impl eframe::App for PrintGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("side_panel")
            .exact_width(320.0)
            .show(ctx, |ui: &mut egui::Ui| {
                let settings_before = self.printer_settings.clone();
                egui::ComboBox::from_label("Printer")
                    .width(200.0)
                    .selected_text(self.printer_settings.printer_name.0.clone())
                    .show_ui(ui, |ui| {
                        for printer in self.printers.values() {
                            ui.selectable_value(
                                &mut self.printer_settings.printer_name.0,
                                printer.name.0.clone(),
                                printer.name.0.clone(),
                            );
                        }
                    });

                let printer = self
                    .printers
                    .get(&self.printer_settings.printer_name.0)
                    .unwrap();
                let media_types: Vec<printer::PaperType> =
                    printer.paper_types.iter().cloned().collect();

                egui::ComboBox::from_label("Media-Type")
                    .width(200.0)
                    .selected_text(self.printer_settings.media_type.0.clone())
                    .show_ui(ui, |ui| {
                        for media_type in media_types.iter() {
                            ui.selectable_value(
                                &mut self.printer_settings.media_type,
                                media_type.clone(),
                                media_type.0.clone(),
                            );
                        }
                    });

                let medias: Vec<printer::PaperSize> = printer.paper_sizes.iter().cloned().collect();
                egui::ComboBox::from_label("Media")
                    .width(200.0)
                    .selected_text(self.printer_settings.media_size.0.clone())
                    .show_ui(ui, |ui| {
                        for media in medias.iter() {
                            ui.selectable_value(
                                &mut self.printer_settings.media_size,
                                media.clone(),
                                media.0.clone(),
                            );
                        }
                    });

                let dim = match self.printer_settings.media_size.guess_paper_dimensions() {
                    Ok(dim) => format!(
                        "width: {}mm \n height:{}mm \n{}",
                        dim.width_mm,
                        dim.height_mm,
                        if dim.borderless { "(Borderless)" } else { "" }
                    ),
                    Err(_) => "Unknown".to_string(),
                };

                ui.label(format!("Guessed paper dimensions: \n{}", dim));

                if settings_before != self.printer_settings {
                    match self.printer_settings.save() {
                        Ok(_) => info!("Saved printer settings"),
                        Err(_) => error!("Failed to save printer settings"),
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            use egui_extras::{Size, StripBuilder};

            StripBuilder::new(ui)
                .size(Size::remainder().at_least(50.0))
                .size(Size::exact(50.0))
                .vertical(|mut strip| {
                    strip.cell(|ui: &mut egui::Ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            let av = ui.available_height();

                            let table = TableBuilder::new(ui)
                                .striped(true)
                                .resizable(false)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::auto().at_most(240.0)) // thumb
                                .column(Column::remainder().at_least(120.0).clip(true))
                                .column(Column::auto()) // status
                                .column(Column::auto())
                                .min_scrolled_height(0.0)
                                .max_scroll_height(av); // print button

                            table
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("Thumbnail");
                                    });
                                    header.col(|ui| {
                                        ui.strong("File");
                                    });

                                    header.col(|ui| {
                                        ui.strong("Status");
                                    });

                                    header.col(|ui| {
                                        ui.strong("Actions");
                                    });
                                })
                                .body(|mut body| {
                                    for img in self.images.iter_mut() {
                                        img.add_to_table_body(&mut body, |a| {
                                            a.print(&self.host, &self.printer_settings);
                                        });
                                    }
                                });
                        });
                    });
                    strip.cell(|ui: &mut egui::Ui| {
                        let clicked = ui.button("PRINTALL").clicked();
                        if clicked {
                            for file in self.images.iter_mut() {
                                file.print(&self.host, &self.printer_settings);
                            }
                        }
                    });
                });
        });
    }
}

pub fn new_print_ui(host: &Uri, files: &Vec<PathBuf>) {
    let printers = match printer::get_printers(host) {
        Ok(a) => a,
        Err(b) => {
            message_box::error(
                "Failed to get printers".to_string(),
                "err".to_string()
            );
            return;
        },
    };

    if printers.len() == 0 {
        message_box::error(
            "Failed to get printers".to_string(),
            "err".to_string()
        );
        return;
    }

    let printer_settings = printer_settings::load_printer_settings();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "k-print",
        options,
        Box::new(move |_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);

            Ok(Box::<PrintGui>::new(PrintGui {
                printer_settings,
                printers,
                images: files.iter().map(|f| UiImage::new(f.clone())).collect(),
                host: host.clone(),
            }))
        }),
    );
}
