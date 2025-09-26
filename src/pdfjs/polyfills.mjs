import pkg from 'skia-canvas';
const { DOMMatrix, Path2D, Canvas, Image, ImageData, CanvasGradient, CanvasPattern, CanvasRenderingContext2D } = pkg;

globalThis.DOMMatrix = DOMMatrix;
globalThis.Path2D = Path2D;
globalThis.Canvas = Canvas;
globalThis.Image = Image;
globalThis.ImageData = ImageData;
globalThis.CanvasGradient = CanvasGradient;
globalThis.CanvasPattern = CanvasPattern;

// Enhanced CanvasRenderingContext2D to handle PDF.js CanvasElement objects
class EnhancedCanvasRenderingContext2D extends CanvasRenderingContext2D {
  drawImage(image, ...args) {
    // Handle PDF.js CanvasElement objects by converting them to proper Canvas instances
    if (image && typeof image === 'object' && image.constructor.name === 'CanvasElement') {
      // Create a new Canvas with the same dimensions and content
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

      // Call the original drawImage with the converted canvas
      return super.drawImage(canvas, ...args);
    }

    // For regular images and canvases, use the original method
    return super.drawImage(image, ...args);
  }
}

globalThis.CanvasRenderingContext2D = EnhancedCanvasRenderingContext2D;

// Also create a custom Canvas class that returns our enhanced context
class EnhancedCanvas extends Canvas {
  getContext(contextType, ...args) {
    if (contextType === '2d') {
      const ctx = super.getContext(contextType, ...args);
      // Override the drawImage method on the returned context
      const originalDrawImage = ctx.drawImage.bind(ctx);
      ctx.drawImage = function(image, ...drawArgs) {
        if (image && typeof image === 'object' && image.constructor.name === 'CanvasElement') {
          const canvas = new Canvas(image.width, image.height);
          const newCtx = canvas.getContext('2d');

          if (image.getContext) {
            const sourceCtx = image.getContext('2d');
            if (sourceCtx && sourceCtx.getImageData) {
              try {
                const imageData = sourceCtx.getImageData(0, 0, image.width, image.height);
                if (imageData && imageData.data) {
                  const newImageData = new ImageData(imageData.data, imageData.width, imageData.height);
                  newCtx.putImageData(newImageData, 0, 0);
                }
              } catch (e) {
                if (image.toDataURL) {
                  const img = new Image();
                  img.src = image.toDataURL();
                  newCtx.drawImage(img, 0, 0);
                }
              }
            }
          }

          return originalDrawImage(canvas, ...drawArgs);
        }

        return originalDrawImage(image, ...drawArgs);
      };
      return ctx;
    }
    return super.getContext(contextType, ...args);
  }
}

// Override the global Canvas
globalThis.Canvas = EnhancedCanvas;