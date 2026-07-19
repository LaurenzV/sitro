import { strict as assert } from "assert";
import fs from "fs";
import path from "path";
import { getDocument } from "pdfjs-dist/legacy/build/pdf.mjs";

async function renderPDF(pdfPath, outputRoot, scaleFactor) {
    const data = new Uint8Array(fs.readFileSync(pdfPath));

    const loadingTask = getDocument({
        data,
        cMapUrl: "node_modules/pdfjs-dist/cmaps/",
        cMapPacked: true,
        standardFontDataUrl: "node_modules/pdfjs-dist/standard_fonts/",
    });

    try {
        const pdfDocument = await loadingTask.promise;
        const canvasFactory = pdfDocument.canvasFactory;

        for (let pageNum = 1; pageNum <= pdfDocument.numPages; pageNum++) {
            const page = await pdfDocument.getPage(pageNum);
            const viewport = page.getViewport({ scale: scaleFactor });
            const canvasAndContext = canvasFactory.create(viewport.width, viewport.height);
            const renderContext = { canvasContext: canvasAndContext.context, viewport };

            const renderTask = page.render(renderContext);
            await renderTask.promise;

            const image = canvasAndContext.canvas.toBuffer("image/png");
            const outputPath = path.join(outputRoot, `page-${pageNum}.png`);
            fs.writeFileSync(outputPath, image);

            page.cleanup();
            canvasFactory.destroy(canvasAndContext);
        }
    } catch (reason) {
        console.error(reason);
        process.exitCode = 1;
    }
}

const [pdfPath, outputRoot, scale] = process.argv.slice(2);
assert(pdfPath, "No PDF path provided");
assert(outputRoot, "No output root directory provided");
assert(scale, "No scale factor provided");

await renderPDF(pdfPath, outputRoot, parseFloat(scale));
