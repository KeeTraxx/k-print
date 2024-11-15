use crate::{
    printer_settings::{self, PrinterSettings},
    Uri,
};
use egui_extras::{Column, TableBuilder};
use log::{error, info};

use eframe::egui;

use crate::printer;

struct PrintGui {
    printer_settings: PrinterSettings,
    printers: Vec<printer::Printer>,
    files: Vec<String>,
    host: Uri,
}

impl eframe::App for PrintGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::right("side_panel")
            .exact_width(320.0)
            .show(ctx, |ui: &mut egui::Ui| {
                let mut printer_index = self
                    .printers
                    .iter()
                    .position(|p| p.name.0 == self.printer_settings.printer.name.0)
                    .unwrap_or(0);

                egui::ComboBox::from_label("Printer")
                    .width(200.0)
                    .selected_text(self.printer_settings.printer.name.0.clone())
                    .show_index(ui, &mut printer_index, self.printers.len(), |i| {
                        self.printers[i].name.0.clone()
                    });

                let printer = self.printer_settings.printer.clone();
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

                            let mut table = TableBuilder::new(ui)
                                .striped(true)
                                .resizable(false)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
                                .column(Column::remainder().at_least(240.0).clip(true))
                                .column(Column::auto()) // status
                                .column(Column::auto())
                                .min_scrolled_height(0.0)
                                .max_scroll_height(av); // print button

                            table
                                .header(20.0, |mut header| {
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
                                    for file in self.files.clone() {
                                        body.row(50.0, |mut row| {
                                            row.col(|ui| {
                                                ui.label(file);
                                            });
                                            row.col(|ui| {
                                                ui.label("Ready");
                                            });
                                            row.col(|ui| {
                                                ui.button("Print");
                                            });
                                        });
                                    }
                                });
                        });
                    });
                    strip.cell(|ui: &mut egui::Ui| {
                        let clicked = ui.button("PRINTALL").clicked();
                        if clicked {
                            // todo PRINT!!
                            info!("saving settings...");

                            match printer_settings::save_printer_settings(&self.printer_settings) {
                                Ok(_) => info!("Saved!!"),
                                Err(_) => error!("error saving settings!!"),
                            }

                            for file in self.files.clone() {
                                match printer::print_file(
                                    &self.host,
                                    &self.printer_settings.printer.name,
                                    &self.printer_settings.media_size,
                                    &self.printer_settings.media_type,
                                    &file,
                                ) {
                                    Ok(_) => log::info!("Printed {}", file),
                                    Err(_) => log::info!("Error printing {}", file),
                                }
                            }
                        }
                    });
                });
        });
    }
}

pub fn gui_print(host: &Uri, files: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let printers_map = printer::get_printers(host)?;
    let printers: Vec<printer::Printer> = printers_map.values().cloned().collect();

    if printers.len() == 0 {
        return Err(Box::new(printer::PrinterError::NoPrinters));
    }

    let state = printer_settings::load_printer_settings();

    info!("Display GUI for files {:#?}", files);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let res = eframe::run_native(
        "k-print",
        options,
        Box::new(move |_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);

            Ok(Box::<PrintGui>::new(PrintGui {
                printer_settings: state,
                printers: printers,
                files: files.clone(),
                host: host.clone(),
            }))
        }),
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
