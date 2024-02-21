use crate::renderer::{RenderOptions, Renderer};
use std::cmp::min;
use std::path::{Path, PathBuf};
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};
use walkdir::WalkDir;

mod renderer;

fn main() {
    let _ = std::fs::remove_dir_all("test");

    let renderers: Vec<Renderer> = vec![
        Renderer::MupdfRenderer,
        Renderer::PdfiumRenderer,
        Renderer::XpdfRenderer,
        Renderer::QuartzRenderer,
        Renderer::PdfjsRenderer,
    ];

    let root_dir = Path::new("pdf");

    let files: Vec<_> = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".pdf"))
        .collect();

    for entry in files {
        let pdf_path = entry.path();
        let file = std::fs::read(pdf_path).unwrap();

        println!("processing {}", pdf_path.to_string_lossy());

        let mut rendered_pages = vec![];

        for renderer in &renderers {
            println!("{}", renderer.name());
            let result = renderer
                .render(&file, &RenderOptions { scale: 1.0 })
                .unwrap();

            rendered_pages.push(result);
        }

        println!("rendering complete image");

        for i in 0..rendered_pages[0].len() {
            let max_width = *[
                imagesize::blob_size(&rendered_pages[0][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[1][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[2][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[3][i]).unwrap().width,
                imagesize::blob_size(&rendered_pages[4][i]).unwrap().width,
            ]
            .iter()
            .max()
            .unwrap();

            let max_height = *[
                imagesize::blob_size(&rendered_pages[0][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[1][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[2][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[3][i]).unwrap().height,
                imagesize::blob_size(&rendered_pages[4][i]).unwrap().height,
            ]
            .iter()
            .max()
            .unwrap();

            let stroke_width = min(max_width, max_height) as f32 / 75.0;

            let path = {
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, 0.0);
                pb.line_to(max_width as f32 + stroke_width, 0.0);
                pb.line_to(
                    max_width as f32 + stroke_width,
                    max_height as f32 + stroke_width,
                );
                pb.line_to(0.0, max_height as f32 + stroke_width);
                pb.close();
                pb.finish().unwrap()
            };

            let mut cursor = Transform::from_translate(stroke_width / 2.0, stroke_width / 2.0);

            let mut pixmap = Pixmap::new(
                ((max_width as f32 + stroke_width * 2.0) * 5.0) as u32,
                (max_height as f32 + stroke_width * 2.0) as u32,
            )
            .unwrap();

            let mut stroke = Stroke::default();
            stroke.width = stroke_width as f32;

            for j in 0..5 {
                let mut paint = Paint::default();
                let cur_renderer = &renderers[j];
                paint.set_color_rgba8(
                    cur_renderer.color().0,
                    cur_renderer.color().1,
                    cur_renderer.color().2,
                    255,
                );

                pixmap.draw_pixmap(
                    0,
                    0,
                    Pixmap::decode_png(&rendered_pages[j][i]).unwrap().as_ref(),
                    &PixmapPaint::default(),
                    Transform::from_translate(
                        cursor.tx + stroke_width / 2.0,
                        cursor.ty + stroke_width / 2.0,
                    ),
                    None,
                );

                pixmap.stroke_path(&path, &paint, &stroke, cursor, None);

                cursor.tx += max_width as f32 + stroke_width * 2.0;
            }

            let mut dir = PathBuf::from("test");
            dir.push(pdf_path.with_extension("").strip_prefix(root_dir).unwrap());
            let mut path = dir.clone();
            path.push(format!("{}.png", i));
            let _ = std::fs::create_dir_all(dir);
            pixmap.save_png(&path).unwrap();
        }
    }
}
