import { Canvas, Path2D, Image, ImageData, DOMMatrix } from '@napi-rs/canvas';

globalThis.Canvas = Canvas;
globalThis.Path2D = Path2D;
globalThis.Image = Image;
globalThis.ImageData = ImageData;
globalThis.DOMMatrix = DOMMatrix;

// Helper function to convert CanvasElement to proper Canvas
function convertCanvasElement(image) {
  if (image && typeof image === 'object' && image.constructor.name === 'CanvasElement') {
    const canvas = new Canvas(image.width, image.height);
    const ctx = canvas.getContext('2d');

    // Try to copy the image data if available
    if (image.getContext) {
      const sourceCtx = image.getContext('2d');
      if (sourceCtx && sourceCtx.getImageData) {
        try {
          const imageData = sourceCtx.getImageData(0, 0, image.width, image.height);
          // Ensure we have a proper ImageData object
          if (imageData && imageData.data) {
            const newImageData = new ImageData(imageData.data, imageData.width, imageData.height);
            ctx.putImageData(newImageData, 0, 0);
          }
        } catch (e) {
          // If getting image data fails, try to draw the image directly if it has toDataURL
          if (image.toDataURL) {
            const img = new Image();
            img.src = image.toDataURL();
            ctx.drawImage(img, 0, 0);
          }
        }
      }
    }

    return canvas;
  }
  return image;
}

// Simple helper function since we're now using the same canvas library everywhere
function ensurePath2D(path) {
  if (!path || path instanceof Path2D) return path;

  // If it's an array (PDF.js path data), convert it
  if (Array.isArray(path)) {
    const newPath = new Path2D();
    for (let i = 0; i < path.length;) {
      const op = path[i++];
      switch (op) {
        case 0: // moveTo
          newPath.moveTo(path[i++], path[i++]);
          break;
        case 1: // lineTo
          newPath.lineTo(path[i++], path[i++]);
          break;
        case 2: // curveTo
          newPath.bezierCurveTo(path[i++], path[i++], path[i++], path[i++], path[i++], path[i++]);
          break;
        case 3: // closePath
          newPath.closePath();
          break;
      }
    }
    return newPath;
  }

  return path;
}

// @napi-rs/canvas should work directly with PDF.js without additional polyfills