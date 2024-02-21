use image::ImageFormat;
use pdfium_render::pdfium::Pdfium;
use pdfium_render::prelude::PdfRenderConfig;
use std::io::Cursor;
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let input_path = Path::new(args.get(1).ok_or("input path missing")?);
    let output_path = Path::new(args.get(2).ok_or("output path missing")?);
    let scale = args
        .get(3)
        .unwrap_or(&"1".to_string())
        .parse::<f32>()
        .map_err(|_| "invalid scale")?;

    let file = std::fs::read(input_path).map_err(|_| "couldnt read input file")?;

    let pdfium =
        Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name()).unwrap());

    let document = pdfium
        .load_pdf_from_byte_slice(&file, None)
        .map_err(|_| "unable to load pdf document".to_string())?;

    for (count, page) in document.pages().iter().enumerate() {
        let mut output_buffer = Cursor::new(vec![]);
        let image = page
            .render_with_config(&PdfRenderConfig::new().scale_page_by_factor(scale))
            .map_err(|_| "unable to render pdf document")?
            .as_image();
        image
            .write_to(&mut output_buffer, ImageFormat::Png)
            .map_err(|_| "unable to render pdf document")?;

        let real_out_path = output_path
            .to_string_lossy()
            .replace("%d", &(count + 1).to_string());

        std::fs::write(real_out_path, output_buffer.into_inner())
            .map_err(|_| "couldn't write output file")?;
    }

    Ok(())
}
