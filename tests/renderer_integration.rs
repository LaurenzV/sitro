//! Integration tests for PDF renderers.
//!
//! These tests verify that each renderer can successfully render a test PDF
//! and produce non-empty PNG output.

use sitro::{RenderOptions, Renderer};

const TEST_PDF: &[u8] = include_bytes!("../assets/font_cid_1.pdf");

fn test_renderer(renderer: Renderer) {
    let options = RenderOptions::default();
    let result = renderer.render_as_png(TEST_PDF, &options);

    match result {
        Ok(pages) => {
            assert!(!pages.is_empty(), "{} returned no pages", renderer.name());
            for (i, page) in pages.iter().enumerate() {
                assert!(!page.is_empty(), "{} returned empty PNG for page {}", renderer.name(), i);
                assert!(
                    page.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
                    "{} returned invalid PNG for page {} (bad magic bytes)",
                    renderer.name(),
                    i
                );
            }
            println!("{} successfully rendered {} page(s)", renderer.name(), pages.len());
        }
        Err(e) => panic!("{} failed: {}", renderer.name(), e),
    }
}

// Native renderers (no Docker required)

#[test]
fn test_hayro() {
    test_renderer(Renderer::Hayro);
}

#[test]
#[cfg(target_os = "macos")]
fn test_quartz() {
    test_renderer(Renderer::Quartz);
}

#[test]
fn test_pdfium() {
    test_renderer(Renderer::Pdfium);
}

#[test]
fn test_mupdf() {
    test_renderer(Renderer::Mupdf);
}

#[test]
fn test_poppler() {
    test_renderer(Renderer::Poppler);
}

#[test]
fn test_ghostscript() {
    test_renderer(Renderer::Ghostscript);
}

#[test]
fn test_pdfbox() {
    test_renderer(Renderer::Pdfbox);
}

#[test]
fn test_pdfjs() {
    test_renderer(Renderer::Pdfjs);
}

#[test]
fn test_serenity() {
    test_renderer(Renderer::Serenity);
}
