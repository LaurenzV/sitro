import { strict as assert } from "assert";
import { getDocument } from "pdfjs-dist/build/pdf.mjs";
import fs from "fs";
import path from "path";

class NodeCanvasFactory {
    create(width, height) {
        assert(width > 0 && height > 0, "Invalid canvas size");
        const canvas = new Canvas(width, height);
        const context = canvas.getContext("2d");
        return { canvas, context };
    }

    reset(canvasAndContext, width, height) {
        assert(canvasAndContext.canvas, "Canvas is not specified");
        assert(width > 0 && height > 0, "Invalid canvas size");
        canvasAndContext.canvas.width = width;
        canvasAndContext.canvas.height = height;
    }

    destroy(canvasAndContext) {
        assert(canvasAndContext.canvas, "Canvas is not specified");
        canvasAndContext.canvas.width = 0;
        canvasAndContext.canvas.height = 0;
        canvasAndContext.canvas = null;
        canvasAndContext.context = null;
    }
}

const canvasFactory = new NodeCanvasFactory();

async function renderPDF(pdfPath, outputRoot, scaleFactor) {
    const data = new Uint8Array(fs.readFileSync(pdfPath));
    const loadingTask = getDocument({
        data,
        cMapUrl: "node_modules/pdfjs-dist/cmaps/",
        cMapPacked: true,
        standardFontDataUrl: "node_modules/pdfjs-dist/standard_fonts/",
        canvasFactory,
    });

    try {
        const pdfDocument = await loadingTask.promise;

        for (let pageNum = 1; pageNum <= pdfDocument.numPages; pageNum++) {
            const page = await pdfDocument.getPage(pageNum);
            const viewport = page.getViewport({ scale: scaleFactor });
            const canvasAndContext = canvasFactory.create(viewport.width, viewport.height);
            const renderContext = { canvasContext: canvasAndContext.context, viewport };

            await page.render(renderContext).promise;

            const image = canvasAndContext.canvas.toBuffer('image/png');
            const outputPath = path.join(outputRoot, `page-${pageNum}.png`);
            fs.writeFileSync(outputPath, image);

            page.cleanup();
        }
    } catch (reason) {
        console.error(reason);
    }
}

const [pdfPath, outputRoot, scale] = process.argv.slice(2);
assert(pdfPath, "No PDF path provided");
assert(outputRoot, "No output root directory provided");
assert(scale, "No scale factor provided");

renderPDF(pdfPath, outputRoot, parseFloat(scale));