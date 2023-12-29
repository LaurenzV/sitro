use crate::renderer::render_with_pdfium;

mod renderer;

fn main() {
    let file = std::fs::read("pdf_reference.pdf").unwrap();
    let result = render_with_pdfium(&file).unwrap();

    for (page, res) in result.iter().enumerate() {
        std::fs::write(format!("out/res-{}.png", page), res).unwrap();
    }
}
