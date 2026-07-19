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

Docker must be installed and running. Sitro automatically uses the Docker image tagged with the same version as the crate and pulls it when it is not available locally:

```text
vallaris/sitro-backends:<crate-version>
```

Set `SITRO_DOCKER_IMAGE` to override the image. The Quartz and Hayro backends run natively with no additional setup.
*/

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(dead_code)]

mod renderer;
pub use renderer::*;
