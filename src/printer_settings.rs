use std::{env, fs};

use serde::{Deserialize, Serialize};

use crate::printer;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PrinterSettings {
    pub printer: printer::Printer,
    pub media_type: printer::PaperType,
    pub media_size: printer::PaperSize,
}
impl PrinterSettings {
    /// Returns the default `PrinterSettings` instance.
    ///
    /// This function creates and returns a `PrinterSettings` object with default
    /// values for the printer, media type, and media size. It is used when no
    /// existing configuration is found, providing a basic setup for printing.
    fn default() -> PrinterSettings {
        let printers = printer::get_printers(&"ipp://localhost:631".parse().unwrap()).unwrap();
        let printer = printers.values().next().unwrap().clone();
        PrinterSettings {
            printer: printer.clone(),
            media_type: printer.paper_types.iter().next().unwrap().clone(),
            media_size: printer.paper_sizes.iter().next().unwrap().clone(),
        }
    }

    /// Checks if the current settings are valid.
    ///
    /// This function checks all the settings in the `PrinterSettings` object,
    /// and returns `true` if all of them are valid, and `false` otherwise.
    ///
    fn is_valid(&self) -> bool {
        let printers = printer::get_printers(&"ipp://localhost:631".parse().unwrap()).unwrap();

        if printers.contains_key(&self.printer.name.0) == false {
           return false;
        }

        let printer = printers.get(&self.printer.name.0).unwrap();

        if printer.has_paper_size(&self.media_size) == false {
            return false;
        }

        if printer.has_paper_type(&self.media_type) == false {
            return false;
        }

        true
    }
}

pub(crate) fn load_printer_settings() -> PrinterSettings {
    fs::exists(settings_file_path())
        .and_then(|_| fs::read_to_string(settings_file_path()))
        .map(|str| toml::from_str::<PrinterSettings>(&str).unwrap())
        .unwrap_or_else(|_| PrinterSettings::default())
}

pub(crate) fn save_printer_settings(
    printer_settings: &PrinterSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = toml::to_string(printer_settings)?;

    fs::write(settings_file_path(), config)?;

    println!("Saved printer settings to {}", settings_file_path());

    Ok(())
}

fn settings_file_path() -> String {
    let config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = env::var("HOME").unwrap();
        format!("{}/.config", home_dir)
    });

    format!("{}/printer-settings.toml", config_dir)
}
