use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::{env, fs};
use tempdir::TempDir;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

/// The options that should be applied when rendering a PDF to a pixmap.
#[derive(Copy, Clone)]
pub struct RenderOptions {
    /// By how much the original size should be scaled.
    pub scale: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

/// A page rendered as a PNG image.
pub type RenderedPage = Vec<u8>;
/// A document rendered as PNG images.
pub type RenderedDocument = Vec<RenderedPage>;

/// A PDF backend used to render a PDF. Each backend calls a command-line
/// utility in the background
pub enum Renderer {
    /// The pdfium renderer.
    Pdfium,
    /// The mupdf renderer.
    Mupdf,
    /// The xpdf renderer.
    Xpdf,
    /// The quartz renderer.
    QuartzRenderer,
    /// The pdf.js renderer.
    PdfjsRenderer,
    /// The pdfbox renderer.
    PdfboxRenderer,
}

impl Renderer {
    pub fn name(&self) -> String {
        match self {
            Renderer::Pdfium => "pdfium".to_string(),
            Renderer::Mupdf => "mupdf".to_string(),
            Renderer::Xpdf => "xpdf".to_string(),
            Renderer::QuartzRenderer => "quartz".to_string(),
            Renderer::PdfjsRenderer => "pdfjs".to_string(),
            Renderer::PdfboxRenderer => "pdfbox".to_string(),
        }
    }

    pub(crate) fn color(&self) -> (u8, u8, u8) {
        match self {
            Renderer::Pdfium => (79, 184, 35),
            Renderer::Mupdf => (34, 186, 184),
            Renderer::Xpdf => (227, 137, 20),
            Renderer::QuartzRenderer => (234, 250, 60),
            Renderer::PdfjsRenderer => (48, 17, 207),
            Renderer::PdfboxRenderer => (237, 38, 98),
        }
    }

    pub(crate) fn render_as_pixmap(
        &self,
        buf: &[u8],
        options: &RenderOptions,
        border_width: Option<f32>,
    ) -> Result<Vec<Pixmap>, String> {
        let pages = self.render_as_png(buf, options)?;
        let Some(border_width) = border_width else {
            return pages
                .iter()
                .map(|page| {
                    Pixmap::decode_png(page).map_err(|_| "unable to generate pixmap".to_string())
                })
                .collect();
        };

        let mut pixmaps = vec![];

        for page in &pages {
            let decoded = Pixmap::decode_png(page).unwrap();
            let width = imagesize::blob_size(&page).unwrap().width as f32;
            let height = imagesize::blob_size(&page).unwrap().height as f32;
            let border_width = min(width as u32, height as u32) as f32 * border_width;

            let actual_width = width + border_width;
            let actual_height = height + border_width;

            let path = {
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, 0.0);
                pb.line_to(width, 0.0);
                pb.line_to(width, height);
                pb.line_to(0.0, height);
                pb.close();
                pb.finish().unwrap()
            };

            let mut pixmap = Pixmap::new(actual_width as u32, actual_height as u32).unwrap();

            let mut stroke = Stroke::default();
            stroke.width = border_width;

            let mut paint = Paint::default();
            paint.set_color_rgba8(self.color().0, self.color().1, self.color().2, 255);

            pixmap.draw_pixmap(
                0,
                0,
                decoded.as_ref(),
                &PixmapPaint::default(),
                Transform::from_translate(border_width, border_width),
                None,
            );

            pixmap.stroke_path(
                &path,
                &paint,
                &stroke,
                Transform::from_translate(border_width / 2.0, border_width / 2.0),
                None,
            );
            pixmaps.push(pixmap);
        }

        Ok(pixmaps)
    }

    /// Render a PDF file as a sequence of PDF files, using the specified renderer.
    pub fn render_as_png(&self, buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
        match self {
            Renderer::Pdfium => render_pdfium(buf, options),
            Renderer::Mupdf => render_mupdf(buf, options),
            Renderer::Xpdf => render_xpdf(buf, options),
            Renderer::QuartzRenderer => render_quartz(buf, options),
            Renderer::PdfjsRenderer => render_pdfjs(buf, options),
            Renderer::PdfboxRenderer => render_pdfbox(buf, options),
        }
    }
}

/// Render a PDF file using pdfium.
pub fn render_pdfium(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, dir: &Path| {
        Command::new(env::var("PDFIUM_BIN").unwrap())
            .arg(&input_path)
            .arg(PathBuf::from(dir).join("out-%d.png"))
            .arg((options.scale).to_string())
            .output()
            .map_err(|e| format!("{}: {}", "failed to run renderer", e))
    };

    let out_file_pattern = r"(?m)out-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

/// Render a PDF file using mupdf.
pub fn render_mupdf(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, dir: &Path| {
        Command::new(env::var("MUPDF_BIN").unwrap())
            .arg("draw")
            .arg("-r")
            .arg((72.0 * options.scale).to_string())
            .arg("-o")
            .arg(PathBuf::from(dir).join("out-%d.png"))
            .arg(&input_path)
            .output()
            .map_err(|e| e.to_string())
    };

    let out_file_pattern = r"(?m)out-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

/// Render a PDF file using xpdf.
pub fn render_xpdf(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, dir: &Path| {
        Command::new(env::var("XPDF_BIN").unwrap())
            .arg("-r")
            .arg((72.0 * options.scale).to_string())
            .arg(&input_path)
            .arg(&dir)
            .output()
            .map_err(|e| format!("{}: {}", "failed to run renderer", e))
    };

    let out_file_pattern = r"(?m)-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

/// Render a PDF file using quartz.
pub fn render_quartz(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, dir: &Path| {
        Command::new(env::var("QUARTZ_BIN").unwrap())
            .arg(&input_path)
            .arg(&dir)
            .arg(options.scale.to_string())
            .output()
            .map_err(|e| format!("{}: {}", "failed to run renderer", e))
    };

    let out_file_pattern = r"(?m)-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

/// Render a PDF file using pdf.js.
pub fn render_pdfjs(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, dir: &Path| {
        Command::new("node")
            .arg(env::var("PDFJS_BIN").unwrap())
            .arg(&input_path)
            .arg(&dir)
            .arg(options.scale.to_string())
            .output()
            .map_err(|e| format!("{}: {}", "failed to run renderer", e))
    };

    let out_file_pattern = r"(?m)-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

/// Render a PDF file using pdfbox.
pub fn render_pdfbox(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let command = |input_path: &Path, _: &Path| {
        let res = Command::new("java")
            .arg("-jar")
            .arg(env::var("PDFBOX_BIN").unwrap())
            .arg("render")
            .arg("-format")
            .arg("png")
            .arg("-i")
            .arg(&input_path)
            .arg("-dpi")
            .arg(format!("{}", 72.0 * options.scale))
            .output()
            .map_err(|e| format!("{}: {}", "failed to run renderer", e));
        eprintln!("{:?}", res);
        return res;
    };

    let out_file_pattern = r"(?m)-(\d+).png";

    render_via_cli(buf, command, out_file_pattern)
}

fn render_via_cli<F>(buf: &[u8], command_fn: F, out_file_pattern: &str) -> Result<RenderedDocument, String>
where
    F: Fn(&Path, &Path) -> Result<Output, String>,
{
    let dir = TempDir::new("sitro").unwrap();
    let input_path = dir.path().join("file.pdf");
    let mut input_file = File::create(&input_path).unwrap();
    input_file.write(buf).unwrap();

    let mut output_dir = PathBuf::from(dir.path());
    output_dir.push("");

    let _ = command_fn(&input_path, &output_dir)?;

    let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir.path())
        .map_err(|_| "")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| {
                    let captures = regex::Regex::new(out_file_pattern)
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
