use core::fmt;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Cursor;

use ipp::prelude::IppClient;

use ipp::prelude::*;
use thiserror::Error;

use crate::image::crop;

#[derive(Debug, Clone)]
pub struct Printer {
    pub name: PrinterName,
    pub printer_uri: PrinterUri,
    pub paper_sizes: HashSet<PaperSize>,
    pub paper_types: HashSet<PaperType>,
}

impl Printer {
    fn has_paper_size(&self, paper_size: &PaperSize) -> bool {
        self.paper_sizes.contains(paper_size)
    }

    fn has_paper_type(&self, paper_type: &PaperType) -> bool {
        self.paper_types.contains(paper_type)
    }
}

impl fmt::Display for Printer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printer (--printer): {}", self.name.0)?;
        writeln!(f, "  Media (--media):")?;
        for paper_size in self.paper_sizes.iter() {
            print!("    {}", paper_size);
            match paper_size.guess_paper_dimensions() {
                Ok(dim) => {
                    let borderless = if dim.borderless { " (Borderless)" } else { "" };
                    print!(
                        "    {}mm x {}mm {}",
                        dim.width_mm, dim.height_mm, borderless
                    );
                }
                Err(_) => print!(""),
            };
            writeln!(f, "")?;
        }

        writeln!(f, "  Media-Types (--media-type):")?;
        for paper_type in self.paper_types.iter() {
            writeln!(f, "    {}", paper_type)?;
        }

        writeln!(f, "\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterName(pub String);
impl fmt::Display for PrinterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterUri(pub String);
impl fmt::Display for PrinterUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PaperSize(pub String);
impl fmt::Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PaperSize {
    pub fn guess_paper_dimensions(&self) -> Result<PaperDimension, Box<dyn std::error::Error>> {
        let width_mm;
        let height_mm;
        if self.0.contains("a4") || self.0.contains("a-4") {
            width_mm = 210.0;
            height_mm = 297.0;
        } else if self.0.contains("b5") {
            width_mm = 182.0;
            height_mm = 257.0;
        } else if self.0.contains("a6") {
            width_mm = 105.0;
            height_mm = 148.0;
        } else if self.0.contains("na_letter") {
            width_mm = inch_to_mm(8.5);
            height_mm = inch_to_mm(11.0);
        } else if self.0.contains("4x6in") {
            width_mm = inch_to_mm(4.0);
            height_mm = inch_to_mm(6.0);
        } else if self.0.contains("3.5x5in") {
            width_mm = inch_to_mm(3.5);
            height_mm = inch_to_mm(5.0);
        } else {
            return Err(Box::new(PrinterError::CouldNotGuessPaperDimensions(
                self.0.clone(),
            )));
        }

        let borderless = self.0.contains("full") || self.0.contains("om_t");

        Ok(PaperDimension {
            width_mm,
            height_mm,
            borderless,
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PaperType(pub String);
impl fmt::Display for PaperType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct PaperDimension {
    pub width_mm: f32,
    pub height_mm: f32,
    pub borderless: bool,
}

impl PaperDimension {
    pub fn to_pixel_dimensions(&self, dpi: f32) -> (u32, u32) {
        (
            mm_to_inch(self.width_mm * dpi) as u32,
            mm_to_inch(self.height_mm * dpi) as u32,
        )
    }
}

pub struct IppJobAccepted {
    pub job_uri: Uri,
    pub job_state_enum: i32,
    pub job_id: i32,
    pub job_state_reasons: String,
    pub job_state_message: String,
}

#[derive(Debug, Error)]
enum PrinterError {
    #[error("No printers found")]
    NoPrinters,
    #[error("Specified printer {0} not found!")]
    SpecifiedPrinterNotFound(PrinterName),
    #[error("Specified paper size {0} not found!")]
    SpecifiedPaperSizeNotFound(PaperSize),
    #[error("Specified paper type {0} not found!")]
    SpecifiedPaperTypeNotFound(PaperType),
    #[error("Can't guess paper type from name: {0}")]
    CouldNotGuessPaperDimensions(String),
    #[error("IPP Error: Status Code: {0}")]
    IppError(StatusCode),
}

pub fn get_printers(host: &Uri) -> Result<HashMap<String, Printer>, Box<dyn std::error::Error>> {
    let client = IppClient::new(host.clone());

    let op = IppOperationBuilder::cups().get_printers();
    let resp = client.send(op)?;

    if resp.header().status_code().is_success() {
        let printers: HashMap<_, _> = resp
            .attributes()
            .groups_of(DelimiterTag::PrinterAttributes)
            .into_iter()
            // .map(|p| {println!("{:#?}", p); p})
            .map(|p| Printer {
                name: PrinterName(
                    p.attributes()[IppAttribute::PRINTER_NAME]
                        .value()
                        .to_string(),
                ),
                printer_uri: PrinterUri(
                    p.attributes()[IppAttribute::PRINTER_URI_SUPPORTED]
                        .value()
                        .to_string(),
                ),
                paper_sizes: HashSet::from_iter(
                    p.attributes()[IppAttribute::MEDIA_SUPPORTED]
                        .value()
                        .into_iter()
                        .map(|f| PaperSize(f.to_string())),
                ),
                paper_types: p.attributes()[IppAttribute::MEDIA_TYPE_SUPPORTED]
                    .value()
                    .into_iter()
                    .map(|f| PaperType(f.to_string()))
                    .collect(),
            })
            .map(|p| (p.name.0.clone(), p))
            .collect();

        Ok(printers)
    } else {
        Err(Box::new(PrinterError::NoPrinters))
    }
}

pub fn get_printer(
    host: &Uri,
    printer_name: &PrinterName,
) -> Result<Printer, Box<dyn std::error::Error>> {
    let printers = get_printers(host)?;
    match printers.get(&printer_name.0) {
        Some(printer) => Ok(printer.clone()),
        None => Err(Box::new(PrinterError::SpecifiedPrinterNotFound(
            printer_name.clone(),
        ))),
    }
}

pub fn print_file(
    host: &Uri,
    printer_name: &PrinterName,
    paper_size: &PaperSize,
    paper_type: &PaperType,
    file: &String,
) -> Result<Vec<IppJobAccepted>, Box<dyn std::error::Error>> {
    let printer = get_printer(&host, &printer_name)?;

    if !printer.has_paper_size(paper_size) {
        return Err(Box::new(PrinterError::SpecifiedPaperSizeNotFound(
            paper_size.clone(),
        )));
    }

    if !printer.has_paper_type(paper_type) {
        return Err(Box::new(PrinterError::SpecifiedPaperTypeNotFound(
            paper_type.clone(),
        )));
    }

    let uri = printer.printer_uri.0.parse()?;
    let client = IppClient::new(uri);

    let (w, h) = paper_size
        .guess_paper_dimensions()?
        .to_pixel_dimensions(300.0);
    let img = crop(file, w, h)?;
    let mut c = Cursor::new(Vec::new());
    img.write_to(&mut c, image::ImageFormat::Png)?;
    c.set_position(0);
    img.save("out.png")?;
    let payload = IppPayload::new(c);

    let mut builder = IppOperationBuilder::print_job(client.uri().clone(), payload);

    // set args
    builder = builder.attribute(IppAttribute::new("media", paper_size.clone().0.parse()?));

    let mut col: BTreeMap<String, IppValue> = BTreeMap::new();
    col.insert(
        "media-type".to_string(),
        IppValue::Keyword(paper_type.clone().0),
    );
    builder = builder.attribute(IppAttribute::new("media-col", IppValue::Collection(col)));

    builder = builder.attribute(IppAttribute::new("print-quality", IppValue::Enum(5)));

    let resp = client.send(builder.build())?;

    if resp.header().status_code().is_success() {
        let result: Vec<IppJobAccepted> = resp
            .attributes()
            .groups_of(DelimiterTag::JobAttributes)
            .into_iter()
            .map(|j| {
                // let j = format!("{:#?}", j);
                // j.attributes()[IppAttribute::JOB_ID].value();
                IppJobAccepted {
                    job_uri: j.attributes()[IppAttribute::JOB_URI]
                        .value()
                        .to_string()
                        .parse()
                        .unwrap(),
                    job_id: j.attributes()[IppAttribute::JOB_ID]
                        .value()
                        .clone()
                        .into_integer()
                        .unwrap(),
                    job_state_enum: j.attributes()[IppAttribute::JOB_STATE]
                        .value()
                        .as_enum()
                        .unwrap()
                        .clone(),
                    job_state_reasons: j.attributes()[IppAttribute::JOB_STATE_REASONS]
                        .value()
                        .as_keyword()
                        .unwrap()
                        .clone(),
                    job_state_message: j.attributes()["job-state-message"].value().to_string(),
                }
            })
            .collect();
        Ok(result)
    } else {
        Err(Box::new(PrinterError::IppError(
            resp.header().status_code(),
        )))
    }
}

fn inch_to_mm(arg: f32) -> f32 {
    arg * 25.4
}

fn mm_to_inch(arg: f32) -> f32 {
    arg / 25.4
}
