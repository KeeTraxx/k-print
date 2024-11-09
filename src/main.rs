mod image;
mod printer;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            printer::print_file(&uri, &printer_name, &paper_size, &paper_type, &file)?;
        }
        Commands::PrinterInfo { printer } => {
            
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
    }
    Ok(())
}
