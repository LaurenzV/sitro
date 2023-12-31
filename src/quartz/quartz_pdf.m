#import <Foundation/Foundation.h>
#import <Quartz/Quartz.h>
#import <CoreServices/CoreServices.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        if (argc != 4) {
            NSLog(@"Usage: PDFToPNG <path to PDF file> <output root directory> <scale factor>");
            return 1;
        }

        NSString *pdfPath = [NSString stringWithUTF8String:argv[1]];
        NSString *outputRoot = [NSString stringWithUTF8String:argv[2]];  // Root directory for output images
        float scaleFactor = atof(argv[3]);

        // Ensure the output directory exists
        NSFileManager *fileManager = [NSFileManager defaultManager];
        if (![fileManager fileExistsAtPath:outputRoot isDirectory:nil]) {
            NSLog(@"Output directory does not exist.");
            return 1;
        }

        NSURL *pdfUrl = [NSURL fileURLWithPath:pdfPath];
        CGPDFDocumentRef pdf = CGPDFDocumentCreateWithURL((__bridge CFURLRef) pdfUrl);

        if (!pdf) {
            NSLog(@"Can't open the PDF.");
            return 1;
        }

        size_t numPages = CGPDFDocumentGetNumberOfPages(pdf);

        for (size_t pageNum = 1; pageNum <= numPages; pageNum++) {
            CGPDFPageRef page = CGPDFDocumentGetPage(pdf, pageNum);
            if (!page) {
                NSLog(@"Can't read page %zu.", pageNum);
                continue;
            }

            CGRect pageRect = CGPDFPageGetBoxRect(page, kCGPDFMediaBox);
            pageRect.size.width *= scaleFactor;
            pageRect.size.height *= scaleFactor;

            CGContextRef context = CGBitmapContextCreate(NULL, pageRect.size.width, pageRect.size.height, 8, 0, CGColorSpaceCreateDeviceRGB(), kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
            if (!context) {
                NSLog(@"Failed to create graphics context.");
                continue;
            }

            CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0);
            CGContextFillRect(context, pageRect);
            CGContextScaleCTM(context, scaleFactor, scaleFactor);
            CGContextDrawPDFPage(context, page);

            CGImageRef imageRef = CGBitmapContextCreateImage(context);
            if (!imageRef) {
                NSLog(@"Failed to create image from context.");
                CGContextRelease(context);
                continue;
            }

            // Construct the output path using the root directory and page number
            NSString *fileName = [[pdfPath lastPathComponent] stringByDeletingPathExtension];
            NSString *outputPath = [outputRoot stringByAppendingPathComponent:[NSString stringWithFormat:@"%@-page-%zu.png", fileName, pageNum]];
            CFURLRef url = (__bridge CFURLRef)[[NSURL alloc] initFileURLWithPath:outputPath];

            CGImageDestinationRef destination = CGImageDestinationCreateWithURL(url, (__bridge CFStringRef)UTTypePNG.identifier, 1, NULL);
            if (!destination) {
                NSLog(@"Failed to create image destination.");
                CGImageRelease(imageRef);
                CGContextRelease(context);
                continue;
            }

            CGImageDestinationAddImage(destination, imageRef, nil);
            if (!CGImageDestinationFinalize(destination)) {
                NSLog(@"Failed to write image to %@", outputPath);
            } else {
                NSLog(@"Converted page %zu of %@ to PNG format at %@", pageNum, pdfPath, outputPath);
            }

            CGContextRelease(context);
            CGImageRelease(imageRef);
        }
        CGPDFDocumentRelease(pdf);
    }
    return 0;
}
