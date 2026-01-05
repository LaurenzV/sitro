use crate::renderer::{Backend, RenderOptions, RENDER_INSTANCE};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::path::{Path, PathBuf};
use tiny_skia::{Pixmap, PixmapPaint, Transform};
use walkdir::WalkDir;

mod renderer;

fn main() {
    let _ = std::fs::remove_dir_all("test");

    let backends: Vec<Backend> = vec![
        Backend::Mupdf,
        Backend::Ghostscript,
        Backend::Pdfium,
        Backend::Poppler,
        Backend::Quartz,
        Backend::Pdfjs,
        Backend::Pdfbox,
        Backend::Hayro,
        Backend::Serenity,
    ];

    let root_dir = Path::new("pdf");

    let files: Vec<_> = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".pdf"))
        .collect();
    
    let instance = RENDER_INSTANCE
        .as_ref()
        .unwrap();

    let options = RenderOptions { scale: 1.75 };

    files.par_iter().for_each(|entry| {
        let pdf_path = entry.path();
        let file = std::fs::read(pdf_path).unwrap();

        let rendered_pages: Vec<_> = backends
            .iter()
            .map(|backend| {
                println!(
                    "rendering {} with {}",
                    pdf_path.to_string_lossy(),
                    backend.name()
                );
                instance
                    .render_as_pixmap(backend, &file, &options, Some(1.0 / 50.0))
                    .unwrap()
            })
            .collect();

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

            for j in 0..backends.len() {
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
            dir.push(
                pdf_path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            );
            let mut path = dir.clone();
            path.push(format!(
                "{}-{}.png",
                pdf_path.file_stem().unwrap().to_str().unwrap(),
                i
            ));
            let _ = std::fs::create_dir_all(dir);
            pixmap.save_png(&path).unwrap();
        }
    });
}
