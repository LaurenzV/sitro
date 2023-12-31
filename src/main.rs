use crate::renderer::{MupdfRenderer, RenderOptions, Renderer, PdfiumRenderer};

mod renderer;

fn main() {
    let file = std::fs::read("pdf_small.pdf").unwrap();

    let renderers: Vec<Box<dyn Renderer>> = vec![Box::from(MupdfRenderer::new()),
                                                 Box::from(PdfiumRenderer::new())];

    for renderer in renderers {
        let result = renderer
            .render(&file, &RenderOptions { scale: 3.5 })
            .unwrap();

        for (page, res) in result.iter().enumerate() {
            std::fs::write(format!("out/{}/res-{}.png", renderer.name(), page), res).unwrap();
        }
    }

}
