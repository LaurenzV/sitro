use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::{env, fs};
use tempdir::TempDir;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

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
type SitroResult = Result<Vec<RenderedPage>, String>;

pub enum Renderer {
    PdfiumRenderer,
    MupdfRenderer,
    XpdfRenderer,
    QuartzRenderer,
    PdfjsRenderer,
    PdfboxRenderer,
}

impl Renderer {
    pub fn name(&self) -> String {
        match self {
            Renderer::PdfiumRenderer => "pdfium".to_string(),
            Renderer::MupdfRenderer => "mupdf".to_string(),
            Renderer::XpdfRenderer => "xpdf".to_string(),
            Renderer::QuartzRenderer => "quartz".to_string(),
            Renderer::PdfjsRenderer => "pdfjs".to_string(),
            Renderer::PdfboxRenderer => "pdfbox".to_string(),
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Renderer::PdfiumRenderer => (79, 184, 35),
            Renderer::MupdfRenderer => (34, 186, 184),
            Renderer::XpdfRenderer => (227, 137, 20),
            Renderer::QuartzRenderer => (234, 250, 60),
            Renderer::PdfjsRenderer => (48, 17, 207),
            Renderer::PdfboxRenderer => (237, 38, 98),
        }
    }

    pub fn render_as_pixmap(
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

    pub fn render_as_png(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        match self {
            Renderer::PdfiumRenderer => self.render_pdfium(buf, options),
            Renderer::MupdfRenderer => self.render_mupdf(buf, options),
            Renderer::XpdfRenderer => self.render_xpdf(buf, options),
            Renderer::QuartzRenderer => self.render_quartz(buf, options),
            Renderer::PdfjsRenderer => self.render_pdfjs(buf, options),
            Renderer::PdfboxRenderer => self.render_pdfbox(buf, options),
        }
    }

    fn render_pdfium(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new(env::var("PDFIUM_BIN").unwrap())
                .arg(&input_path)
                .arg(PathBuf::from(dir).join("out-%d.png"))
                .arg((options.scale).to_string())
                .output()
                .map_err(|_| "failed to run renderer".to_string())
        };

        let out_file_pattern = r"(?m)out-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_mupdf(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new(env::var("MUPDF_BIN").unwrap())
                .arg("draw")
                .arg("-r")
                .arg((72.0 * options.scale).to_string())
                .arg("-o")
                .arg(PathBuf::from(dir).join("out-%d.png"))
                .arg(&input_path)
                .output()
                .map_err(|_| "failed to run renderer".to_string())
        };

        let out_file_pattern = r"(?m)out-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_xpdf(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new(env::var("XPDF_BIN").unwrap())
                .arg("-r")
                .arg((72.0 * options.scale).to_string())
                .arg(&input_path)
                .arg(&dir)
                .output()
                .map_err(|_| "failed to run renderer".to_string())
        };

        let out_file_pattern = r"(?m)-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_quartz(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new(env::var("QUARTZ_BIN").unwrap())
                .arg(&input_path)
                .arg(&dir)
                .arg(options.scale.to_string())
                .output()
                .map_err(|_| "failed to run renderer".to_string())
        };

        let out_file_pattern = r"(?m)-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_pdfjs(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new("node")
                .arg(env::var("PDFJS_BIN").unwrap())
                .arg(&input_path)
                .arg(&dir)
                .arg(options.scale.to_string())
                .output()
                .map_err(|_| "failed to run renderer".to_string())
        };

        let out_file_pattern = r"(?m)-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_pdfbox(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
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
                .map_err(|_| "failed to run renderer".to_string());
            println!("{:?}", res);
            return res;
        };

        let out_file_pattern = r"(?m)-(\d+).png";

        self.render_via_cli(buf, command, out_file_pattern)
    }

    fn render_via_cli<F>(&self, buf: &[u8], command_fn: F, out_file_pattern: &str) -> SitroResult
    where
        F: Fn(&Path, &Path) -> Result<Output, String>,
    {
        let dir = TempDir::new("sitro").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        let mut output_dir = PathBuf::from(dir.path());
        output_dir.push("");

        let output = command_fn(&input_path, &output_dir)?;

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
}
