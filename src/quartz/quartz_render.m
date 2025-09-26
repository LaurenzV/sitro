#import <Foundation/Foundation.h>
#import <Quartz/Quartz.h>
#import <CoreServices/CoreServices.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

typedef struct {
    size_t start;
    size_t end;
} PageRange;

PageRange parsePageRange(const char* rangeStr, size_t totalPages) {
    PageRange result = {0, 0};

    if (!rangeStr) {
        // Default: all pages (1-indexed)
        result.start = 1;
        result.end = totalPages;
        return result;
    }

    NSString *range = [NSString stringWithUTF8String:rangeStr];

    if ([range containsString:@"-"]) {
        // Range format: "start-end"
        NSArray *parts = [range componentsSeparatedByString:@"-"];
        if ([parts count] != 2) {
            NSLog(@"Invalid page range format, use start-end");
            result.start = 0; // Invalid marker
            return result;
        }

        NSInteger start = [[parts objectAtIndex:0] integerValue];
        NSInteger end = [[parts objectAtIndex:1] integerValue];

        if (start < 1 || end < 1) {
            NSLog(@"Page numbers must be 1-indexed");
            result.start = 0; // Invalid marker
            return result;
        }

        if (start > totalPages || end > totalPages) {
            NSLog(@"Page range exceeds document pages (1-%zu)", totalPages);
            result.start = 0; // Invalid marker
            return result;
        }

        if (start > end) {
            NSLog(@"Start page must be <= end page");
            result.start = 0; // Invalid marker
            return result;
        }

        result.start = start;
        result.end = end;
    } else {
        // Single page
        NSInteger page = [range integerValue];
        if (page < 1) {
            NSLog(@"Page numbers must be 1-indexed");
            result.start = 0; // Invalid marker
            return result;
        }
        if (page > totalPages) {
            NSLog(@"Page %ld exceeds document pages (1-%zu)", (long)page, totalPages);
            result.start = 0; // Invalid marker
            return result;
        }
        result.start = page;
        result.end = page;
    }

    return result;
}

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        if (argc < 4 || argc > 5) {
            NSLog(@"Usage: PDFToPNG <path to PDF file> <output root directory> <scale factor> [page range]");
            NSLog(@"Page range examples: 3 (single page), 2-5 (pages 2 through 5)");
            return 1;
        }

        NSString *pdfPath = [NSString stringWithUTF8String:argv[1]];
        NSString *outputRoot = [NSString stringWithUTF8String:argv[2]];  // Root directory for output images
        float scaleFactor = atof(argv[3]);
        const char *pageRangeStr = (argc == 5) ? argv[4] : NULL;

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

        // Parse the page range
        PageRange range = parsePageRange(pageRangeStr, numPages);
        if (range.start == 0) {
            // Invalid range, error already logged
            CGPDFDocumentRelease(pdf);
            return 1;
        }

        for (size_t pageNum = range.start; pageNum <= range.end; pageNum++) {
            CGPDFPageRef page = CGPDFDocumentGetPage(pdf, pageNum);
            if (!page) {
                NSLog(@"Can't read page %zu.", pageNum);
                continue;
            }

            // Get the media box (full page area) of the current page
            CGRect mediaBox = CGPDFPageGetBoxRect(page, kCGPDFMediaBox);

            // Calculate the new page dimensions based on the scale factor
            CGSize scaledSize = CGSizeMake(mediaBox.size.width * scaleFactor, mediaBox.size.height * scaleFactor);

            // Create a bitmap context for rendering the page
            CGContextRef context = CGBitmapContextCreate(NULL,
                                                         scaledSize.width,
                                                         scaledSize.height,
                                                         8, // bits per component
                                                         0, // automatic bytes per row
                                                         CGColorSpaceCreateDeviceRGB(),
                                                         kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
            if (!context) {
                NSLog(@"Failed to create graphics context.");
                continue;
            }

            // Fill the background with white
            CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0);
            CGContextFillRect(context, CGRectMake(0, 0, scaledSize.width, scaledSize.height));

            // Scale the context to the correct size
            CGContextScaleCTM(context, scaleFactor, scaleFactor);

            // Translate the context so that the origin aligns with the media box
            CGContextTranslateCTM(context, -mediaBox.origin.x, -mediaBox.origin.y);

            // Render the PDF page into the context
            CGContextDrawPDFPage(context, page);

            // Create a CGImage from the context
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

            // Add the image to the destination and finalize the PNG file
            CGImageDestinationAddImage(destination, imageRef, nil);
            if (!CGImageDestinationFinalize(destination)) {
                NSLog(@"Failed to write image to %@", outputPath);
            }

            // Clean up
            CGContextRelease(context);
            CGImageRelease(imageRef);
        }

        // Release the PDF document
        CGPDFDocumentRelease(pdf);
    }
    return 0;
}
