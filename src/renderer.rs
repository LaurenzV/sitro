use std::io::Cursor;
use image::ImageFormat;
use lazy_static::lazy_static;
use pdfium_render::prelude::{Pdfium, PdfRenderConfig};

lazy_static! {
    static ref PDFIUM_INSTANCE: Pdfium = Pdfium::new(
    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        "lib/",
    ))
        .unwrap(),
);
}

pub fn render_with_pdfium(buf: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let document = PDFIUM_INSTANCE
        .load_pdf_from_byte_slice(buf, None)
        .map_err(|_| "unable to load pdf document")?;

    let mut images = vec![];

    for (counter, page) in document.pages().iter().enumerate() {
        println!("Processing page {:?}", counter);
        let mut output_buffer = Cursor::new(vec![]);
        let image = page
            .render_with_config(&PdfRenderConfig::new())
            .map_err(|_| "unable to render pdf document")?
            .as_image();
        image.write_to(&mut output_buffer, ImageFormat::Png)
            .map_err(|_| "unable to render pdf document")?;



        images.push(output_buffer.into_inner());
    }

    Ok(images)
}