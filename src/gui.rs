use std::ops::Index;

use crate::Uri;
use egui::InnerResponse;
use log::info;

use eframe::egui;

use crate::printer;

struct UserState {
    printer: printer::Printer,
    media_type: printer::PaperType,
    media_size: printer::PaperSize,
}

pub fn gui_print(host: &Uri, files: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let printers_map = printer::get_printers(host)?;
    let printers: Vec<printer::Printer> = printers_map.values().cloned().collect();

    if (printers.len() == 0) {
        return Err(Box::new(printer::PrinterError::NoPrinters));
    }

    let mut state = UserState {
        printer: printers[0].clone(),
        media_type: printers[0].paper_types.iter().next().unwrap().clone(),
        media_size: printers[0].paper_sizes.iter().next().unwrap().clone(),
    };

    info!("Display GUI for files {:#?}", files);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let res = eframe::run_simple_native("k-print", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let mut printer_index = printers
                .iter()
                .position(|p| p.name.0 == state.printer.name.0)
                .unwrap_or(0);

            egui::ComboBox::from_label("Printer")
                .width(200.0)
                .selected_text(state.printer.name.0.clone())
                .show_index(ui, &mut printer_index, printers.len(), |i| {
                    printers[i].name.0.clone()
                });

            let printer = state.printer.clone();
            let media_types: Vec<printer::PaperType> =
                printer.paper_types.iter().cloned().collect();

            egui::ComboBox::from_label("Media-Type")
                .width(200.0)
                .selected_text(state.media_type.0.clone())
                .show_ui(ui, |ui| {
                    for (i, media_type) in media_types.iter().enumerate() {
                        ui.selectable_value(&mut state.media_type, media_type.clone(), media_type.0.clone());
                    }
                });

            ui.label(format!(
                "Printer: {} media_type: {}",
                printer.name.0,
                state.media_type.0
            ));
        });
    });

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
