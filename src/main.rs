use std::path::{Path, PathBuf};
use crate::renderer::{MupdfRenderer, PdfiumRenderer, RenderOptions, Renderer, XpdfRenderer, QuartzRenderer, PdfjsRenderer};

mod renderer;

fn main() {
    let file = std::fs::read("test.pdf").unwrap();

    let _ = std::fs::remove_dir_all("out");

    let renderers: Vec<Box<dyn Renderer>> = vec![
        Box::from(MupdfRenderer::new()),
        Box::from(PdfiumRenderer::new()),
        Box::from(XpdfRenderer::new()),
        Box::from(QuartzRenderer::new()),
        Box::from(PdfjsRenderer::new())
    ];

    for renderer in renderers {
        println!("rendering with {}", renderer.name());
        let result = renderer
            .render(&file, &RenderOptions { scale: 2.0 })
            .unwrap();

        for (page, res) in result.iter().enumerate() {
            let mut dir = PathBuf::from("out");
            dir.push(renderer.name());
            let mut path = dir.clone();
            path.push(format!("res-{}.png", page));
            let _ = std::fs::create_dir_all(dir);
            std::fs::write(path, res).unwrap();
        }
    }
}
