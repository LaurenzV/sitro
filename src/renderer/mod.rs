use hayro::vello_cpu::color::AlphaColor;
use hayro::{InterpreterSettings, Pdf, RenderSettings};
use std::cmp::min;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, LazyLock};
use std::{env, fs};
use tempdir::TempDir;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

#[cfg(target_os = "macos")]
mod quartz;

const DOCKER_IMAGE: &str = "vallaris/sitro-backends:latest";

/// The global render instance.
pub static RENDER_INSTANCE: LazyLock<Option<Renderer>> = LazyLock::new(|| Renderer::new().ok());

/// The renderer used to render PDFs with different backends.
pub struct Renderer {
    container_id: String,
    work_dir: TempDir,
    #[allow(dead_code)]
    child: Child, // Kept alive to maintain stdin pipe; container dies when this drops
}

impl Renderer {
    fn new() -> Result<Self, String> {
        let docker_image =
            env::var("SITRO_DOCKER_IMAGE").unwrap_or_else(|_| DOCKER_IMAGE.to_string());
        let work_dir = TempDir::new("sitro").map_err(|e| e.to_string())?;

        // Start container attached to stdin - when our process dies, stdin closes,
        // cat exits, and --rm cleans up the container
        let child = Command::new("docker")
            .args(["run", "--rm", "-i", "--entrypoint", "cat", "-v"])
            .arg(format!("{}:/work", work_dir.path().to_string_lossy()))
            .arg(&docker_image)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to start docker: {e}"))?;

        // Poll until the container is running
        let container_id = loop {
            let output = Command::new("docker")
                .args([
                    "ps",
                    "-q",
                    "-l",
                    "--filter",
                    &format!("ancestor={docker_image}"),
                ])
                .output()
                .map_err(|e| format!("failed to get container id: {e}"))?;

            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !id.is_empty() {
                break id;
            }
        };

        Ok(Self {
            container_id,
            work_dir,
            child,
        })
    }

    /// Render a PDF using the specified backend.
    pub fn render(
        &self,
        backend: &Backend,
        buf: &[u8],
        options: &RenderOptions,
    ) -> Result<RenderedDocument, String> {
        // For native backends, handle directly without Docker
        match backend {
            #[cfg(target_os = "macos")]
            Backend::Quartz => return quartz::render(buf, options),
            Backend::Hayro => return render_hayro(buf, options),
            _ => {}
        }

        // Create a unique subdirectory for this render to allow parallel execution
        let render_id = uuid::Uuid::new_v4().to_string();
        let render_dir = self.work_dir.path().join(&render_id);
        fs::create_dir_all(&render_dir).map_err(|e| e.to_string())?;

        // Write input PDF
        fs::write(render_dir.join("file.pdf"), buf).map_err(|e| e.to_string())?;

        // Execute render command
        let output = Command::new("docker")
            .args(["exec", &self.container_id, "/opt/bin/entrypoint.sh"])
            .args([
                &backend.name(),
                &options.scale.to_string(),
                &format!("/work/{render_id}"),
            ])
            .output()
            .map_err(|e| format!("docker exec failed: {e}"))?;

        if !output.status.success() {
            let _ = fs::remove_dir_all(&render_dir);
            return Err(format!(
                "render failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let result = read_output_files(&render_dir, r"(?m)out-(\d+).png");
        let _ = fs::remove_dir_all(&render_dir);
        result
    }

    /// Render a PDF and return pixmaps with optional border.
    pub fn render_as_pixmap(
        &self,
        backend: &Backend,
        buf: &[u8],
        options: &RenderOptions,
        border_width: Option<f32>,
    ) -> Result<Vec<Pixmap>, String> {
        let pages = self.render(backend, buf, options)?;
        render_pages_to_pixmaps(&pages, backend.color(), border_width)
    }
}

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

/// A PDF rendering backend.
///
/// Each backend calls a command-line utility in the background (via Docker),
/// except for Quartz and Hayro which run natively.
#[derive(Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Backend {
    /// The pdfium backend (via Docker).
    Pdfium,
    /// The mupdf backend (via Docker).
    Mupdf,
    /// The poppler backend (via Docker).
    Poppler,
    /// The quartz backend (macOS only, runs natively).
    #[cfg(target_os = "macos")]
    Quartz,
    /// The pdf.js backend (via Docker).
    Pdfjs,
    /// The pdfbox backend (via Docker).
    Pdfbox,
    /// The ghostscript backend (via Docker).
    Ghostscript,
    /// The hayro backend (runs natively).
    Hayro,
    /// The serenity backend (SerenityOS LibPDF, via Docker).
    Serenity,
}

impl Backend {
    /// Get the name of the backend.
    pub fn name(&self) -> String {
        match self {
            Backend::Pdfium => "pdfium".to_string(),
            Backend::Mupdf => "mupdf".to_string(),
            Backend::Poppler => "poppler".to_string(),
            #[cfg(target_os = "macos")]
            Backend::Quartz => "quartz".to_string(),
            Backend::Pdfjs => "pdfjs".to_string(),
            Backend::Pdfbox => "pdfbox".to_string(),
            Backend::Ghostscript => "ghostscript".to_string(),
            Backend::Hayro => "hayro".to_string(),
            Backend::Serenity => "serenity".to_string(),
        }
    }

    pub(crate) fn color(&self) -> (u8, u8, u8) {
        match self {
            Backend::Pdfium => (79, 184, 35),
            Backend::Mupdf => (34, 186, 184),
            Backend::Poppler => (227, 137, 20),
            #[cfg(target_os = "macos")]
            Backend::Quartz => (234, 250, 60),
            Backend::Pdfjs => (48, 17, 207),
            Backend::Pdfbox => (237, 38, 98),
            Backend::Ghostscript => (235, 38, 218),
            Backend::Hayro => (57, 212, 116),
            Backend::Serenity => (148, 87, 235),
        }
    }
}

/// Helper function to convert rendered PNG pages to pixmaps with optional borders.
fn render_pages_to_pixmaps(
    pages: &[RenderedPage],
    color: (u8, u8, u8),
    border_width: Option<f32>,
) -> Result<Vec<Pixmap>, String> {
    let Some(border_width) = border_width else {
        return pages
            .iter()
            .map(|page| {
                Pixmap::decode_png(page).map_err(|_| "unable to generate pixmap".to_string())
            })
            .collect();
    };

    let mut pixmaps = vec![];

    for page in pages {
        let decoded = Pixmap::decode_png(page).unwrap();
        let width = imagesize::blob_size(page).unwrap().width as f32;
        let height = imagesize::blob_size(page).unwrap().height as f32;
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
        paint.set_color_rgba8(color.0, color.1, color.2, 255);

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
