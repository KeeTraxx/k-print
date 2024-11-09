use image::DynamicImage;



pub fn crop(file_path : &String, to_width: u32, to_height: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut img = image::open(file_path)?;

    println!("w:{} h:{}", img.width(), img.height());
    
    if img.width() > img.height() {
        img = img.rotate90();
    }

    Ok(img.resize_to_fill(to_width, to_height, image::imageops::FilterType::Lanczos3))
}