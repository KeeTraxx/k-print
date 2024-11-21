use std::{env, fs};

use serde::{Deserialize, Serialize};

use crate::printer::{self, *};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct PrinterSettings {
    pub printer_name: PrinterName,
    pub media_type: PaperType,
    pub media_size: PaperSize,
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
            printer_name: printer.name.clone(),
            media_type: printer.paper_types.iter().next().unwrap().clone(),
            media_size: printer.paper_sizes.iter().next().unwrap().clone(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = toml::to_string(self)?;

        fs::write(settings_file_path(), config)?;

        println!("Saved printer settings to {}", settings_file_path());

        Ok(())
    }
}

pub(crate) fn load_printer_settings() -> PrinterSettings {
    fs::exists(settings_file_path())
        .and_then(|_| fs::read_to_string(settings_file_path()))
        .map(|str| toml::from_str::<PrinterSettings>(&str))
        .unwrap()
        .unwrap_or_else(|_| PrinterSettings::default())
}

fn settings_file_path() -> String {
    let config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = env::var("HOME").unwrap();
        format!("{}/.config", home_dir)
    });

    format!("{}/printer-settings.toml", config_dir)
}
