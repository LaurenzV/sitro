use crate::renderer::{RenderOptions, Renderer};
use std::path::{Path, PathBuf};
use tiny_skia::{Pixmap, PixmapPaint, Transform};
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
        println!("processing {}", pdf_path.to_string_lossy());
        let file = std::fs::read(pdf_path).unwrap();

        let rendered_pages = renderers
            .iter()
            .map(|renderer| {
                println!("rendering with {}", renderer.name());
                renderer
                    .render_as_pixmap(&file, &RenderOptions { scale: 2.5 }, Some(1.0 / 50.0))
                    .unwrap()
            })
            .collect::<Vec<_>>();

        for i in 0..rendered_pages[0].len() {
            let width = rendered_pages
                .iter()
                .map(|pixmaps| pixmaps[i].width())
                .sum();
            let height = rendered_pages
                .iter()
                .map(|pixmaps| pixmaps[i].height())
                .max()
                .unwrap();

            let mut pixmap = Pixmap::new(width, height).unwrap();

            let mut cursor = 0.0;

            for j in 0..5 {
                let cur_pixmap = rendered_pages[j][i].as_ref();
                pixmap.draw_pixmap(
                    0,
                    0,
                    cur_pixmap,
                    &PixmapPaint::default(),
                    Transform::from_translate(cursor, 0.0),
                    None,
                );

                cursor += cur_pixmap.width() as f32;
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
