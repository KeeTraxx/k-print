mod image;
mod printer;
mod gui;
mod printer_settings;

use env_logger::{Builder, Env};
use ipp::prelude::*;
use clap::{Parser, Subcommand};
use printer::{get_printer, PaperSize, PaperType, PrinterName};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "ipp://localhost:631")]
    ipp_host: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Print {
        #[arg(short, long)]
        printer: String,

        #[arg(short = 's', long)]
        media_size: String,

        #[arg(short = 't', long)]
        media_type: String,

        #[arg(short, long)]
        file: String,
    },

    PrinterInfo {
        #[arg(short, long)]
        printer: Option<String>,
    },

    Gui {
        #[arg()]
        files: Vec<String>
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let uri: Uri = cli.ipp_host.parse()?;

    match &cli.command {
        Commands::Print {
            printer,
            media_size,
            media_type,
            file,
        } => {
            let printer_name = PrinterName(printer.trim().to_string());
            let paper_size = PaperSize(media_size.trim().to_string());
            let paper_type = PaperType(media_type.trim().to_string());
            let file = file.trim().to_string();
            let ipp_jobs  = printer::print_file(&uri, &printer_name, &paper_size, &paper_type, &file)?;

            for job in ipp_jobs {
                println!("Printer accepted print job id: {} uri: {}", job.job_id, job.job_uri);
            }

        }
        Commands::PrinterInfo { printer } => {
            log::info!("Printer info");
            if printer.is_none() {
                let printers = printer::get_printers(&uri)?;
                for (_name, p) in printers.iter() {
                    println!("{}", p);
                }
            } else {
                let p = get_printer(&uri, &PrinterName(printer.as_ref().unwrap().clone()))?;
                println!("{}", &p)
            }
        }
        Commands::Gui { files } => {
            // log::info!("Gui");
            // println!("Files: {:#?}", files);
            gui::gui_print(&uri, &files.clone())?;
        },
    }
    Ok(())
}
