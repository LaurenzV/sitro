use crate::renderer::{MupdfRenderer, RenderOptions, Renderer, PdfiumRenderer, XpdfRenderer};

mod renderer;

fn main() {
    let file = std::fs::read("idefix.pdf").unwrap();

    let renderers: Vec<Box<dyn Renderer>> = vec![Box::from(MupdfRenderer::new()),
                                                 Box::from(PdfiumRenderer::new()),
    Box::from(XpdfRenderer::new())
    ];

    for renderer in renderers {
        let result = renderer
            .render(&file, &RenderOptions { scale: 1.0 })
            .unwrap();

        for (page, res) in result.iter().enumerate() {
            std::fs::write(format!("out/{}/res-{}.png", renderer.name(), page), res).unwrap();
        }
    }

}
