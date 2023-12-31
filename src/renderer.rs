use image::ImageFormat;
use pdfium_render::prelude::{PdfRenderConfig, Pdfium};
use std::fs;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;

#[derive(Copy, Clone)]
pub struct RenderOptions {
    pub scale: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

type RenderedPage = Vec<u8>;

pub trait Renderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String>;
}

pub struct PdfiumRenderer {
    instance: Pdfium,
}

impl PdfiumRenderer {
    pub fn new() -> Self {
        Self {
            instance: Pdfium::new(
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("lib/"))
                    .unwrap(),
            ),
        }
    }
}

impl Renderer for PdfiumRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let document = self
            .instance
            .load_pdf_from_byte_slice(buf, None)
            .map_err(|_| "unable to load pdf document")?;

        let mut images = vec![];

        for page in document.pages().iter() {
            let mut output_buffer = Cursor::new(vec![]);
            let image = page
                .render_with_config(&PdfRenderConfig::new().scale_page_by_factor(options.scale))
                .map_err(|_| "unable to render pdf document")?
                .as_image();
            image
                .write_to(&mut output_buffer, ImageFormat::Png)
                .map_err(|_| "unable to render pdf document")?;
            images.push(output_buffer.into_inner());
        }

        Ok(images)
    }
}

pub struct MupdfRenderer {}

impl MupdfRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for MupdfRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let dir = TempDir::new("mupdf").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        Command::new("mutool")
            .arg("draw")
            .arg("-r")
            .arg((72.0 * options.scale).to_string())
            .arg("-o")
            .arg(dir.path().join("out-%d.png"))
            .arg(&input_path)
            .output()
            .map_err(|_| "failed to run mupdf")?;

        let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir.path())
            .map_err(|_| "")?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        let captures = regex::Regex::new(r"(?m)out-(\d+).png")
                            .unwrap()
                            .captures(name)?;
                        let num_str = captures.get(1)?;
                        let num: i32 = num_str.as_str().parse().ok()?;
                        Some((num, path.clone()))
                    })
            })
            .collect::<Vec<_>>();

        out_files.sort_by_key(|e| e.0);

        let out_files = out_files.iter().map(|e| fs::read(&e.1).unwrap()).collect();

        Ok(out_files)
    }
}
