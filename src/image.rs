use image::{DynamicImage, GenericImageView, Pixel, Rgb, Rgba, RgbaImage};



pub fn crop(file_path : &String, to_width: u32, to_height: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut img = image::open(file_path)?;

    if img.width() > img.height() {
        img = img.rotate90();
    }

    let new_image = remove_alpha(&img, Rgb([255, 255, 255]));
    
    Ok(new_image.resize_to_fill(to_width, to_height, image::imageops::FilterType::Lanczos3))
}

    /// Takes an image and removes its alpha channel. The resulting image is a new instance of image::RgbaImage.
    ///
    /// This function is useful for printing images, since many printers don't support alpha channels.
    ///
pub fn remove_alpha(img: &DynamicImage, bg_color: Rgb<u8>) -> DynamicImage {

    let mut new_image = RgbaImage::new(img.width(), img.height());
    for (x,y, pixel) in img.pixels() {
        let rgba = pixel.to_rgba();
        let alpha = rgba[3];

        let new_pixel = if alpha < 255 {
            Rgba([
                ((rgba[0] as u16 * alpha as u16 + bg_color[0] as u16 * (255 - alpha as u16)) / 255) as u8,
                ((rgba[1] as u16 * alpha as u16 + bg_color[1] as u16 * (255 - alpha as u16)) / 255) as u8,
                ((rgba[2] as u16 * alpha as u16 + bg_color[2] as u16 * (255 - alpha as u16)) / 255) as u8,
                255,
            ])
        } else {
            rgba
        };

        new_image.put_pixel(x, y, new_pixel);
    }
    
    DynamicImage::from(new_image)

}