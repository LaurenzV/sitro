# sitro

A Rust library for rendering PDFs with multiple backends to compare output across different PDF engines.

## Backends

Sitro supports seven rendering backends:

| Backend | Used by | Platform |
|---------|---------|----------|
| pdfium | Google Chrome | Docker |
| mupdf | - | Docker |
| poppler | Evince, GNOME | Docker |
| ghostscript | - | Docker |
| pdfbox | Apache | Docker |
| pdf.js | Firefox | Docker |
| quartz | Apple Preview | macOS native |

## Setup

All you have to do is pull the docker image, the rest should happen automatically.

```bash
docker pull ghcr.io/laurenzv/sitro-backends:latest
```

## Note
Note that this crate has been built for personal purposes and has not been reviewed carefully (including for example
the code for rendering via the Quartz framework). I don't recommend using this crate for production use cases.
