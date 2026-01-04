#!/bin/bash
set -e

BACKEND="$1"
SCALE="${2:-1.0}"
INPUT_PDF="/work/file.pdf"

if [ -z "$BACKEND" ]; then
    echo "Usage: entrypoint.sh <backend> [scale]" >&2
    echo "Backends: pdfium, mupdf, poppler, ghostscript, pdfbox, pdfjs" >&2
    exit 1
fi

if [ ! -f "$INPUT_PDF" ]; then
    echo "Error: Input PDF not found at $INPUT_PDF" >&2
    exit 1
fi

DPI=$(awk "BEGIN {printf \"%.0f\", $SCALE * 72}")

case "$BACKEND" in
    pdfium)
        /opt/bin/pdfium "$INPUT_PDF" "/work/out-%d.png" "$SCALE"
        ;;
    mupdf)
        /opt/bin/mutool draw -q -r "$DPI" -o "/work/out-%d.png" "$INPUT_PDF"
        ;;
    poppler)
        pdftoppm -r "$DPI" -png "$INPUT_PDF" "/work/out"
        ;;
    ghostscript)
        /opt/bin/gs -dNOPAUSE -dBATCH -sDEVICE=png16m \
            -dGraphicsAlphaBits=4 -dTextAlphaBits=4 -r"$DPI" \
            -sOutputFile="/work/out-%d.png" "$INPUT_PDF"
        ;;
    pdfbox)
        java -jar /opt/bin/pdfbox.jar render -format png -i "$INPUT_PDF" -dpi "$DPI"
        for f in /work/file-*.png; do
            [ -f "$f" ] && mv "$f" "/work/out-$(basename "$f" | sed 's/file-\([0-9]*\)\.png/\1/').png"
        done
        ;;
    pdfjs)
        cd /opt/pdfjs && node pdfjs_render.mjs "$INPUT_PDF" "/work" "$SCALE"
        for f in /work/page-*.png; do
            [ -f "$f" ] && mv "$f" "/work/out-$(basename "$f" | sed 's/page-\([0-9]*\)\.png/\1/').png"
        done
        ;;
    *)
        echo "Error: Unknown backend '$BACKEND'" >&2
        echo "Backends: pdfium, mupdf, poppler, ghostscript, pdfbox, pdfjs" >&2
        exit 1
        ;;
esac
