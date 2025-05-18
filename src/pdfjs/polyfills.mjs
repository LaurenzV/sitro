import pkg from 'skia-canvas';
const { DOMMatrix, Path2D, Canvas, Image, ImageData, CanvasGradient, CanvasPattern, CanvasRenderingContext2D } = pkg;

globalThis.DOMMatrix = DOMMatrix;
globalThis.Path2D = Path2D;
globalThis.Canvas = Canvas;
globalThis.Image = Image;
globalThis.ImageData = ImageData;
globalThis.CanvasGradient = CanvasGradient;
globalThis.CanvasPattern = CanvasPattern;
globalThis.CanvasRenderingContext2D = CanvasRenderingContext2D;