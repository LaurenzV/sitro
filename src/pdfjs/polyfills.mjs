import pkg from 'skia-canvas';
const { DOMMatrix, Path2D, Canvas } = pkg;

globalThis.DOMMatrix = DOMMatrix;
globalThis.Path2D = Path2D;
globalThis.Canvas = Canvas;