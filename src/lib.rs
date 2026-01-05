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
| serenity | SerenityOS | Docker |
| quartz | Apple Preview | macOS native |
| hayro | - | native |

# Setup

Pull the Docker image:

```bash
docker pull vallaris/sitro-backends
```

That's it. The Quartz and Hayro backends run natively with no additional setup.
```
*/

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(dead_code)]

mod renderer;
pub use renderer::*;
