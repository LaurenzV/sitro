use std::path::{Path, PathBuf};
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};
use walkdir::WalkDir;
use crate::renderer::{MupdfRenderer, PdfiumRenderer, RenderOptions, Renderer, XpdfRenderer, QuartzRenderer, PdfjsRenderer};

mod renderer;

fn main() {
    let _ = std::fs::remove_dir_all("test");


    let renderers: Vec<Box<dyn Renderer>> = vec![
        Box::from(MupdfRenderer::new()),
        Box::from(PdfiumRenderer::new()),
        Box::from(XpdfRenderer::new()),
        Box::from(QuartzRenderer::new()),
        Box::from(PdfjsRenderer::new())
    ];

    let root_dir = Path::new("pdf");

    let files: Vec<_> = WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()).collect();

    for entry in files {
        let pdf_path = entry.path();
        let file = std::fs::read(pdf_path).unwrap();

        println!("processing {}", pdf_path.to_string_lossy());

        let mut rendered_pages= vec![];

        for renderer in &renderers {
            println!("{}", renderer.name());
            let result = renderer
                .render(&file, &RenderOptions { scale: 2.0 })
                .unwrap();

            rendered_pages.push(result);
        }

        let mut consolidated_images: Vec<u8> = vec![];

        for i in 0..rendered_pages[0].len() {
            let max_width = *[
                imagesize::blob_size(&rendered_pages[0][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[1][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[2][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[3][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[4][i]).unwrap().width,
            ].iter().max().unwrap();

            let max_height = *[
                imagesize::blob_size(&rendered_pages[0][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[1][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[2][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[3][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[4][i]).unwrap().height,
            ].iter().max().unwrap();

            let stroke_width = 3;

            let path = {
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, 0.0);
                pb.line_to(max_width as f32, 0.0);
                pb.line_to(max_width as f32, max_height as f32);
                pb.line_to(0.0, max_height as f32);
                pb.close();
                pb.finish().unwrap()
            };

            let mut pixmap = Pixmap::new(((max_width + stroke_width) * 5) as u32, (max_height + stroke_width) as u32).unwrap();

            let mut stroke = Stroke::default();
            stroke.width = stroke_width as f32;

            for j in 0..5 {
                let mut paint = Paint::default();
                paint.set_color_rgba8(renderers[j].color().0, renderers[j].color().1, renderers[j].color().2, 255);

                pixmap.draw_pixmap(((max_width * j) + stroke_width) as i32, 0, Pixmap::decode_png(&rendered_pages[j][i]).unwrap().as_ref(),
                                   &PixmapPaint::default(), Transform::default(), None);
                pixmap.stroke_path(&path, &paint, &stroke, Transform::from_translate((max_width * j + stroke_width) as f32, 0.0), None);
            }

            pixmap.save_png("out.png");
        }

        // for (page, res) in result.iter().enumerate() {
        //     let mut dir = PathBuf::from("test");
        //     dir.push(pdf_path.with_extension("").strip_prefix(root_dir).unwrap());
        //     let mut path = dir.clone();
        //     path.push(format!("{}-{}.png", page, renderer.name()));
        //     let _ = std::fs::create_dir_all(dir);
        //     std::fs::write(path, res).unwrap();
        // }
    }
}
