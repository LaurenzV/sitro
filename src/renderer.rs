use std::fs;
use std::fs::File;
use std::io::Write;
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

    fn name(&self) -> String;

    fn color(&self) -> (u8, u8, u8);
}

pub struct PdfiumRenderer {}

impl PdfiumRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for PdfiumRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let dir = TempDir::new("pdfium").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        Command::new("target/release/pdfium")
            .arg(&input_path)
            .arg(dir.path().join("out-%d.png"))
            .arg((options.scale).to_string())
            .output()
            .map_err(|_| "failed to run pdfium")?;

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

    fn name(&self) -> String {
        "pdfium".to_string()
    }

    fn color(&self) -> (u8, u8, u8) {
        (79, 184, 35)
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

    fn name(&self) -> String {
        "mupdf".to_string()
    }

    fn color(&self) -> (u8, u8, u8) {
        (34, 186, 184)
    }
}

pub struct XpdfRenderer {}

impl XpdfRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for XpdfRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let dir = TempDir::new("xpdf").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        // Needed so that trailing slash is added
        let mut dir_path = PathBuf::from(dir.path());
        dir_path.push("");

        let out = Command::new("pdftopng")
            .arg("-r")
            .arg((72.0 * options.scale).to_string())
            .arg(&input_path)
            .arg(&dir_path)
            .output()
            .map_err(|_| "failed to run xpdf")?;

        let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir.path())
            .map_err(|_| "")?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        let captures = regex::Regex::new(r"(?m)-(\d+).png")
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

    fn name(&self) -> String {
        "xpdf".to_string()
    }

    fn color(&self) -> (u8, u8, u8) {
        (227, 137, 20)
    }
}

pub struct QuartzRenderer {}

impl QuartzRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for QuartzRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let dir = TempDir::new("quartz").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        // Needed so that trailing slash is added
        let mut dir_path = PathBuf::from(dir.path());
        dir_path.push("");

        let out = Command::new("src/quartz/quartz_render")
            .arg(&input_path)
            .arg(&dir_path)
            .arg(options.scale.to_string())
            .output()
            .unwrap();

        let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir.path())
            .map_err(|_| "")?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        let captures = regex::Regex::new(r"(?m)-(\d+).png")
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

    fn name(&self) -> String {
        "quartz".to_string()
    }

    fn color(&self) -> (u8, u8, u8) {
        (234, 250, 60)
    }
}

pub struct PdfjsRenderer {}

impl PdfjsRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for PdfjsRenderer {
    fn render(&self, buf: &[u8], options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
        let dir = TempDir::new("pdfjs").unwrap();
        let input_path = dir.path().join("file.pdf");
        let mut input_file = File::create(&input_path).unwrap();
        input_file.write(buf).unwrap();

        // Needed so that trailing slash is added
        let mut dir_path = PathBuf::from(dir.path());
        dir_path.push("");

        let out = Command::new("node")
            .arg("src/pdfjs/pdfjs_render.mjs")
            .arg(&input_path)
            .arg(&dir_path)
            .arg(options.scale.to_string())
            .output()
            .unwrap();

        let mut out_files: Vec<(i32, PathBuf)> = fs::read_dir(dir.path())
            .map_err(|_| "")?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| {
                        let captures = regex::Regex::new(r"(?m)-(\d+).png")
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

    fn name(&self) -> String {
        "pdfjs".to_string()
    }

    fn color(&self) -> (u8, u8, u8) {
        (48, 17, 207)
    }
}
