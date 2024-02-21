use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
type SitroResult = Result<Vec<RenderedPage>, String>;

pub enum Renderer {
    PdfiumRenderer,
    MupdfRenderer,
    XpdfRenderer,
    QuartzRenderer,
    PdfjsRenderer,
}

impl Renderer {
    pub fn name(&self) -> String {
        match self {
            Renderer::PdfiumRenderer => "pdfium".to_string(),
            Renderer::MupdfRenderer => "mupdf".to_string(),
            Renderer::XpdfRenderer => "xpdf".to_string(),
            Renderer::QuartzRenderer => "quartz".to_string(),
            Renderer::PdfjsRenderer => "pdfjs".to_string(),
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Renderer::PdfiumRenderer => (79, 184, 35),
            Renderer::MupdfRenderer => (34, 186, 184),
            Renderer::XpdfRenderer => (227, 137, 20),
            Renderer::QuartzRenderer => (234, 250, 60),
            Renderer::PdfjsRenderer => (48, 17, 207),
        }
    }

    pub fn render(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        match self {
            Renderer::PdfiumRenderer => self.render_pdfium(buf, options),
            Renderer::MupdfRenderer => self.render_mupdf(buf, options),
            Renderer::XpdfRenderer => self.render_xpdf(buf, options),
            Renderer::QuartzRenderer => self.render_quartz(buf, options),
            Renderer::PdfjsRenderer => self.render_pdfjs(buf, options),
        }
    }

    fn render_pdfium(&self, buf: &[u8], options: &RenderOptions) -> SitroResult {
        let command = |input_path: &Path, dir: &Path| {
            Command::new("target/release/pdfium")
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
            Command::new("mutool")
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
            Command::new("pdftopng")
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
            Command::new("src/quartz/quartz_render")
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
                .arg("src/pdfjs/pdfjs_render.mjs")
                .arg(&input_path)
                .arg(&dir)
                .arg(options.scale.to_string())
                .output()
                .map_err(|_| "failed to run renderer".to_string())
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
