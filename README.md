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

Docker must be installed and running. Sitro automatically uses the Docker image tagged with the same version as the crate and pulls it when it is not available locally.

The image name is:

```text
vallaris/sitro-backends:<crate-version>
```

That's it. When accessing the global render instance, sitro will automatically spawn a Docker container that contains the utilities necessary for rendering the PDFs with the given backend.

Set `SITRO_DOCKER_IMAGE` to override the image, for example when testing a local build.

## Publishing the Docker image

Update `version` in `Cargo.toml`, then run:

```bash
./docker/build-and-push.sh
```

The script publishes multi-platform Docker images tagged with both the crate version and `latest`. Dependencies that link to the corresponding Git commit automatically use the versioned image. Version tags must not contain SemVer build metadata (`+...`) because `+` is not valid in a Docker tag.

## Note

Note that this crate has been built for personal purposes and has not been reviewed carefully (including for example the code for rendering via the Quartz framework). I don't recommend using this crate for production use cases.
