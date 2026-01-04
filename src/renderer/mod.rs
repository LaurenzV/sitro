use hayro::vello_cpu::color::AlphaColor;
use hayro::{InterpreterSettings, Pdf, RenderSettings};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::{env, fs};
use tempdir::TempDir;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

#[cfg(target_os = "macos")]
mod quartz;

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
/// utility in the background (via Docker), except for Quartz and Hayro which
/// run natively.
#[derive(Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Renderer {
    /// The pdfium renderer (via Docker).
    Pdfium,
    /// The mupdf renderer (via Docker).
    Mupdf,
    /// The poppler renderer (via Docker).
    Poppler,
    /// The quartz renderer (macOS only, runs natively).
    #[cfg(target_os = "macos")]
    Quartz,
    /// The pdf.js renderer (via Docker).
    Pdfjs,
    /// The pdfbox renderer (via Docker).
    Pdfbox,
    /// The ghostscript renderer (via Docker).
    Ghostscript,
    /// The hayro renderer (runs natively).
    Hayro,
}

impl Renderer {
    /// Get the name of the renderer.
    pub fn name(&self) -> String {
        match self {
            Renderer::Pdfium => "pdfium".to_string(),
            Renderer::Mupdf => "mupdf".to_string(),
            Renderer::Poppler => "poppler".to_string(),
            #[cfg(target_os = "macos")]
            Renderer::Quartz => "quartz".to_string(),
            Renderer::Pdfjs => "pdfjs".to_string(),
            Renderer::Pdfbox => "pdfbox".to_string(),
            Renderer::Ghostscript => "ghostscript".to_string(),
            Renderer::Hayro => "hayro".to_string(),
        }
    }

    pub(crate) fn color(&self) -> (u8, u8, u8) {
        match self {
            Renderer::Pdfium => (79, 184, 35),
            Renderer::Mupdf => (34, 186, 184),
            Renderer::Poppler => (227, 137, 20),
            #[cfg(target_os = "macos")]
            Renderer::Quartz => (234, 250, 60),
            Renderer::Pdfjs => (48, 17, 207),
            Renderer::Pdfbox => (237, 38, 98),
            Renderer::Ghostscript => (235, 38, 218),
            Renderer::Hayro => (57, 212, 116),
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

    /// Render a PDF file as a sequence of PNG files, using the specified renderer.
    pub fn render_as_png(
        &self,
        buf: &[u8],
        options: &RenderOptions,
    ) -> Result<RenderedDocument, String> {
        match self {
            #[cfg(target_os = "macos")]
            Renderer::Quartz => quartz::render(buf, options),
            Renderer::Hayro => render_hayro(buf, options),
            // All other backends run via Docker
            _ => render_via_docker(buf, &self.name(), options.scale),
        }
    }
}

/// Render a PDF file using hayro.
fn render_hayro(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let pdf = Pdf::new(Arc::new(buf.to_vec())).map_err(|e| format!("{:?}", e))?;
    let interpreter_settings = InterpreterSettings::default();

    let render_settings = RenderSettings {
        x_scale: options.scale,
        y_scale: options.scale,
        width: None,
        height: None,
        bg_color: AlphaColor::WHITE,
    };

    pdf.pages()
        .iter()
        .map(|page| {
            hayro::render(page, &interpreter_settings, &render_settings)
                .into_png()
                .map_err(|e| format!("{:?}", e))
        })
        .collect()
}

const DOCKER_IMAGE: &str = "vallaris/sitro-backends:latest";

/// Render a PDF using Docker container.
fn render_via_docker(
    buf: &[u8],
    backend_name: &str,
    scale: f32,
) -> Result<RenderedDocument, String> {
    let dir = TempDir::new("sitro").unwrap();
    let input_path = dir.path().join("file.pdf");
    let mut input_file = File::create(&input_path).unwrap();
    input_file.write_all(buf).unwrap();

    // Get Docker image name from env or use default
    let docker_image = env::var("SITRO_DOCKER_IMAGE").unwrap_or_else(|_| DOCKER_IMAGE.to_string());

    // Run Docker container with volume mount
    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/work", dir.path().to_string_lossy()))
        .arg(&docker_image)
        .arg(backend_name)
        .arg(scale.to_string())
        .output()
        .map_err(|e| format!("failed to run docker: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "docker execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // All backends output out-N.png via entrypoint normalization
    read_output_files(dir.path(), r"(?m)out-(\d+).png")
}

/// Read output PNG files from a directory, matching them against a pattern.
fn read_output_files(dir: &Path, out_file_pattern: &str) -> Result<RenderedDocument, String> {
    let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir)
        .map_err(|_| "failed to read output directory")?
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
