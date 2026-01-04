/*!
Render PDFs with multiple backends to compare output across different PDF engines.

# Backends

| Backend | Used by | Platform |
|---------|---------|----------|
| pdfium | Google Chrome | Docker |
| mupdf | - | Docker |
| poppler | Evince, GNOME | Docker |
| ghostscript | - | Docker |
| pdfbox | Apache | Docker |
| pdf.js | Firefox | Docker |
| quartz | Apple Preview | macOS native |

# Setup

Pull the Docker image:

```bash
docker pull vallaris/sitro-backends
```

That's it. The Quartz backend runs natively on macOS with no additional setup.

# Usage

```rust,ignore
use sitro::{Renderer, RenderOptions};

let pdf_bytes = std::fs::read("document.pdf")?;
let options = RenderOptions { scale: 1.5 };
let pages = Renderer::Pdfium.render_as_png(&pdf_bytes, &options)?;
```
*/

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(dead_code)]

mod renderer;
pub use renderer::*;
