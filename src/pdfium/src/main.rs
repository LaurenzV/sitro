use image::ImageFormat;
use pdfium_render::prelude::*;
use std::io::Cursor;
use std::path::Path;

fn parse_page_range(range_str: &str, total_pages: usize) -> Result<(usize, usize), String> {
    if range_str.contains('-') {
        let parts: Vec<&str> = range_str.split('-').collect();
        if parts.len() != 2 {
            return Err("invalid page range format, use start-end".to_string());
        }

        let start = parts[0].parse::<usize>().map_err(|_| "invalid start page")?;
        let end = parts[1].parse::<usize>().map_err(|_| "invalid end page")?;

        if start < 1 || end < 1 {
            return Err("page numbers must be 1-indexed".to_string());
        }

        if start > total_pages || end > total_pages {
            return Err(format!("page range exceeds document pages (1-{})", total_pages));
        }

        if start > end {
            return Err("start page must be <= end page".to_string());
        }

        Ok((start - 1, end - 1)) // Convert to 0-indexed
    } else {
        let page = range_str.parse::<usize>().map_err(|_| "invalid page number")?;
        if page < 1 {
            return Err("page numbers must be 1-indexed".to_string());
        }
        if page > total_pages {
            return Err(format!("page {} exceeds document pages (1-{})", page, total_pages));
        }
        Ok((page - 1, page - 1)) // Convert to 0-indexed, single page
    }
}

fn main() -> Result<(), String> {
    let pdfium =
        Pdfium::new(Pdfium::bind_to_system_library().map_err(|_| "failed to link to pdfium")?);

    let args: Vec<_> = std::env::args().collect();
    let input_path = Path::new(args.get(1).ok_or("input path missing")?);
    let output_path = Path::new(args.get(2).ok_or("output path missing")?);
    let scale = args
        .get(3)
        .unwrap_or(&"1".to_string())
        .parse::<f32>()
        .map_err(|_| "invalid scale")?;

    let page_range_str = args.get(4);

    let file = std::fs::read(input_path).map_err(|_| "couldnt read input file")?;

    let document = pdfium
        .load_pdf_from_byte_slice(&file, None)
        .map_err(|_| "unable to load pdf document".to_string())?;

    let total_pages = document.pages().len() as usize;
    let (start_page, end_page) = if let Some(range_str) = page_range_str {
        parse_page_range(range_str, total_pages)?
    } else {
        (0, total_pages - 1) // Default: all pages
    };

    for (count, page) in document.pages().iter().enumerate().skip(start_page).take(end_page - start_page + 1) {
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
