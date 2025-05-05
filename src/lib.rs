/*!
# Motivation
The purpose of this crate is to provide a unified interface into rendering a PDF with
different renderers. The PDF specification is complex, and for a PDF producer it's easy to do
things wrong. Usually, just checking one PDF viewer is not enough to ensure that your PDF is
devoid of undefined behavior, because it's possible that an invalid PDF just happens to display
fine in one viewer, but not in others. Because of this, it's vital to check many different PDF
viewers and check that they all display the PDF fine.

# Backends
This crate supports six different rendering backends:
- Quartz (used by Apple Preview)
- pdfium (used by Google Chrome)
- pdf.js (used by Firefox)
- mupdf
- poppler
- pdfbox

By having access to these six renderers, you can be certain to a certain extent that if a PDF
renders fine for all of them, there should be no major issues with the PDF. It's obviously no guarantee,
but it at least gives some more confidence.

Most notably (and regrettably), the crate does not support rendering PDFs with Adobe Acrobat. This
is unfortunate since it's clearly the most important viewer, but I was not able to figure out an easy
way to render a PDF via a CLI, so it's unfortunately not supported for now.

# Setup
Unfortunately, in order to use this crate it's not enough to just add it as a dependency. The way
how `sitro` works is that it calls different command line utilities that render a PDF using some backend.
For each backend you want to use, you need to set an environment variable to point to that renderer.
Below you can find the requirements for each renderer.

## Pdfium
For this renderer, you need to build the rust crate that you can find in `src/pdfium`. In addition
to that, you also need to have the [pdfium libray](https://github.com/bblanchon/pdfium-binaries) on
your system so that it can be linked to statically. Then, you simply need to point the `PDFIUM_BIN`
environment variable to the binary.

## mupdf
For this backend, you simply need the `mutool` utility somewhere on your system and point the
`MUPDF_BIN` environment variable to it.

## poppler
For this backend, you simply need the [pdftoppm](https://man.archlinux.org/man/pdftoppm.1.en)
somewhere on your system and point the `POPPLER_BIN` environment variable to it.

## pdfbox
For this backend, you need to have Java installed on your system. In addition, you need to point
the `PDFBOX_BIN` binary to the `.jar` file.

## pdf.js
For this backend, you need to have node.js on your system. Then you need to check out the
`src/pdfjs` folder on your system and install the required node packages. Then, you simply
need to point the `PDFJS_BIN` environment variable to the location of the `pdfjs_render.mjs` file.

## quartz
This backend will only work on MacOS. You need to check out the `src/quartz` folder and run
`build.sh`. Then, you simply point the `QUARTZ_BIN` environment variable to the `quartz_render`
utility.

# Other notes
Note that this crate isn't in the best "shape" in terms of structure and documentation. The reason
for this is that I mainly use it for personal purposes, so I didn't put a lot of effort into cleaning
it up, and that's why it's also not released on crates.io. Nevertheless, it should still work fine
for anyone who has the exact need of rendering a PDF with different backends.
*/

#![forbid(unsafe_code)]
#![allow(dead_code)]

mod renderer;
pub use renderer::*;
