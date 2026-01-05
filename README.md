# sitro

A Rust library for rendering PDFs with multiple backends to compare output across different PDF engines.

## Backends

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

## Setup

All you need to do is pull this docker image:

```bash
docker pull vallaris/sitro-backends
```

That's it. When accessing the global render instance, sitro will automatically spawn a Docker container that contains the utilities necessary for rendering the PDFs with the given backend.

## Note

Note that this crate has been built for personal purposes and has not been reviewed carefully (including for example the code for rendering via the Quartz framework). I don't recommend using this crate for production use cases.
