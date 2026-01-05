//! Integration tests for PDF renderers.

use sitro::{Backend, RenderOptions, RENDER_INSTANCE};

const TEST_PDF: &[u8] = include_bytes!("../assets/font_cid_1.pdf");

fn test_backend(backend: Backend) {
    let renderer = RENDER_INSTANCE
        .as_ref()
        .expect("Failed to initialize renderer");
    let options = RenderOptions::default();
    let result = renderer.render(&backend, TEST_PDF, &options);

    match result {
        Ok(pages) => {
            assert!(!pages.is_empty(), "{} returned no pages", backend.name());
            for (i, page) in pages.iter().enumerate() {
                assert!(
                    !page.is_empty(),
                    "{} returned empty PNG for page {}",
                    backend.name(),
                    i
                );
                assert!(
                    page.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
                    "{} returned invalid PNG for page {} (bad magic bytes)",
                    backend.name(),
                    i
                );
            }
            println!(
                "{} successfully rendered {} page(s)",
                backend.name(),
                pages.len()
            );
        }
        Err(e) => panic!("{} failed: {}", backend.name(), e),
    }
}

#[test]
fn test_hayro() {
    test_backend(Backend::Hayro);
}

#[test]
#[cfg(target_os = "macos")]
fn test_quartz() {
    test_backend(Backend::Quartz);
}

#[test]
fn test_pdfium() {
    test_backend(Backend::Pdfium);
}

#[test]
fn test_mupdf() {
    test_backend(Backend::Mupdf);
}

#[test]
fn test_poppler() {
    test_backend(Backend::Poppler);
}

#[test]
fn test_ghostscript() {
    test_backend(Backend::Ghostscript);
}

#[test]
fn test_pdfbox() {
    test_backend(Backend::Pdfbox);
}

#[test]
fn test_pdfjs() {
    test_backend(Backend::Pdfjs);
}

#[test]
fn test_serenity() {
    test_backend(Backend::Serenity);
}
