use crate::renderer::{MupdfRenderer, RenderOptions, Renderer, PdfiumRenderer};

mod renderer;

fn main() {
    let file = std::fs::read("pdf_small.pdf").unwrap();

    let renderer = MupdfRenderer::new();
    let result = renderer
        .render(&file, &RenderOptions { scale: 3.5 })
        .unwrap();

    for (page, res) in result.iter().enumerate() {
        std::fs::write(format!("out/mupdf/res-{}.png", page), res).unwrap();
    }

    let renderer = PdfiumRenderer::new();
    let result = renderer
        .render(&file, &RenderOptions { scale: 3.5 })
        .unwrap();

    for (page, res) in result.iter().enumerate() {
        std::fs::write(format!("out/pdfium/res-{}.png", page), res).unwrap();
    }


}
