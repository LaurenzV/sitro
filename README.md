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

Docker must be installed and running. Sitro automatically uses the Docker image pinned to the crate by its multi-platform manifest digest and pulls it when it is not available locally.

That's it. When accessing the global render instance, sitro will automatically spawn a Docker container that contains the utilities necessary for rendering the PDFs with the given backend.

Set `SITRO_DOCKER_IMAGE` to override the image, for example when testing a local build.

## Publishing the Docker image

Update `version` in `Cargo.toml`, then run:

```bash
./docker/build-and-push.sh
```

The script builds and tests a local image before publishing the immutable, multi-platform image tagged with the crate version. It then updates `docker/backend-image.lock` with the manifest digest. Commit the updated lock file before publishing the crate. Dependencies that link to the corresponding Git commit automatically use the pinned image. Version tags must not contain SemVer build metadata (`+...`) because `+` is not valid in a Docker tag.

If the image was pushed but the script was interrupted before updating the lock file, recover it with:

```bash
./docker/build-and-push.sh --sync-lock
```

## Note

Note that this crate has been built for personal purposes and has not been reviewed carefully (including for example the code for rendering via the Quartz framework). I don't recommend using this crate for production use cases.
