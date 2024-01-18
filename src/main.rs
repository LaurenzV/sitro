use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::renderer::{MupdfRenderer, PdfiumRenderer, RenderOptions, Renderer, XpdfRenderer, QuartzRenderer, PdfjsRenderer};

mod renderer;

fn main() {
    // let _ = std::fs::remove_dir_all("test");


    let renderers: Vec<Box<dyn Renderer>> = vec![
        Box::from(MupdfRenderer::new()),
        Box::from(PdfiumRenderer::new()),
        Box::from(XpdfRenderer::new()),
        Box::from(QuartzRenderer::new()),
        Box::from(PdfjsRenderer::new())
    ];

    let root_dir = Path::new("/Users/lstampfl/Programming/GitHub/typst/tests/pdf/visualize");

    let files: Vec<_> = WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()).collect();

    for entry in files {
        let pdf_path = entry.path();
        let file = std::fs::read(pdf_path).unwrap();

        println!("processing {}", pdf_path.to_string_lossy());

        for renderer in &renderers {
            let result = renderer
                .render(&file, &RenderOptions { scale: 2.0 })
                .unwrap();

            for (page, res) in result.iter().enumerate() {
                let mut dir = PathBuf::from("test");
                dir.push(pdf_path.with_extension("").strip_prefix(root_dir).unwrap());
                let mut path = dir.clone();
                path.push(format!("{}-{}.png", page, renderer.name()));
                let _ = std::fs::create_dir_all(dir);
                std::fs::write(path, res).unwrap();
            }
        }
    }
}
