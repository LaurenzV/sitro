//! Native Quartz PDF rendering for macOS.

#![allow(unsafe_code)]

use crate::renderer::{RenderOptions, RenderedDocument, RenderedPage};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::base::kCGBitmapByteOrderDefault;
use core_graphics::color_space::CGColorSpace;
use core_graphics::context::CGContext;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::image::CGImageAlphaInfo;
use foreign_types::ForeignType;
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

#[repr(C)]
struct CGPDFDocument(c_void);
type CGPDFDocumentRef = *const CGPDFDocument;

#[repr(C)]
struct CGPDFPage(c_void);
type CGPDFPageRef = *const CGPDFPage;

#[repr(i32)]
#[allow(dead_code)]
enum CGPDFBox {
    MediaBox = 0,
    CropBox = 1,
    BleedBox = 2,
    TrimBox = 3,
    ArtBox = 4,
}

#[repr(C)]
struct __CFMutableData(c_void);
type CFMutableDataRef = *mut __CFMutableData;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPDFDocumentCreateWithProvider(provider: *const c_void) -> CGPDFDocumentRef;
    fn CGPDFDocumentRelease(document: CGPDFDocumentRef);
    fn CGPDFDocumentGetNumberOfPages(document: CGPDFDocumentRef) -> usize;
    fn CGPDFDocumentGetPage(document: CGPDFDocumentRef, page_number: usize) -> CGPDFPageRef;
    fn CGPDFPageGetBoxRect(page: CGPDFPageRef, box_type: CGPDFBox) -> CGRect;
    fn CGPDFPageGetRotationAngle(page: CGPDFPageRef) -> i32;
    fn CGContextDrawPDFPage(context: *mut c_void, page: CGPDFPageRef);
    fn CGContextSetInterpolationQuality(context: *mut c_void, quality: i32);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataCreateMutable(allocator: *const c_void, capacity: isize) -> CFMutableDataRef;
    fn CFDataGetLength(data: *const c_void) -> isize;
    fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
    fn CFRelease(cf: *const c_void);
}

#[link(name = "ImageIO", kind = "framework")]
extern "C" {
    fn CGImageDestinationCreateWithData(
        data: CFMutableDataRef,
        type_: *const c_void,
        count: usize,
        options: *const c_void,
    ) -> *mut c_void;
    fn CGImageDestinationAddImage(
        dest: *mut c_void,
        image: *const c_void,
        properties: *const c_void,
    );
    fn CGImageDestinationFinalize(dest: *mut c_void) -> bool;
}

const K_CG_INTERPOLATION_HIGH: i32 = 3;

pub fn render(buf: &[u8], options: &RenderOptions) -> Result<RenderedDocument, String> {
    let scale = options.scale;

    let buffer = Arc::new(buf.to_vec());
    let provider = CGDataProvider::from_buffer(buffer);

    let document = unsafe { CGPDFDocumentCreateWithProvider(provider.as_ptr() as *const c_void) };
    if document.is_null() {
        return Err("Failed to create PDF document".to_string());
    }

    let page_count = unsafe { CGPDFDocumentGetNumberOfPages(document) };
    let mut pages: Vec<RenderedPage> = Vec::with_capacity(page_count);

    for page_num in 1..=page_count {
        let page = unsafe { CGPDFDocumentGetPage(document, page_num) };
        if page.is_null() {
            unsafe { CGPDFDocumentRelease(document) };
            return Err(format!("Failed to get page {}", page_num));
        }

        match render_page(page, scale) {
            Ok(png_data) => pages.push(png_data),
            Err(e) => {
                unsafe { CGPDFDocumentRelease(document) };
                return Err(e);
            }
        }
    }

    unsafe { CGPDFDocumentRelease(document) };

    Ok(pages)
}

fn render_page(page: CGPDFPageRef, scale: f32) -> Result<RenderedPage, String> {
    let crop_box = unsafe { CGPDFPageGetBoxRect(page, CGPDFBox::CropBox) };
    let rotation = unsafe { CGPDFPageGetRotationAngle(page) };

    let (width, height) =
        if rotation == 90 || rotation == 270 || rotation == -90 || rotation == -270 {
            (crop_box.size.height, crop_box.size.width)
        } else {
            (crop_box.size.width, crop_box.size.height)
        };

    let scaled_width = (width * scale as f64).ceil() as usize;
    let scaled_height = (height * scale as f64).ceil() as usize;

    if scaled_width == 0 || scaled_height == 0 {
        return Err("Invalid page dimensions".to_string());
    }

    let color_space = CGColorSpace::create_device_rgb();
    let bytes_per_row = scaled_width * 4;

    let context = CGContext::create_bitmap_context(
        None,
        scaled_width,
        scaled_height,
        8,
        bytes_per_row,
        &color_space,
        kCGBitmapByteOrderDefault | CGImageAlphaInfo::CGImageAlphaPremultipliedLast as u32,
    );

    context.set_rgb_fill_color(1.0, 1.0, 1.0, 1.0);
    context.fill_rect(CGRect::new(
        &CGPoint::new(0.0, 0.0),
        &CGSize::new(scaled_width as f64, scaled_height as f64),
    ));

    unsafe {
        CGContextSetInterpolationQuality(context.as_ptr() as *mut c_void, K_CG_INTERPOLATION_HIGH);
    }

    context.scale(scale as f64, scale as f64);

    match rotation {
        90 | -270 => {
            context.translate(height, 0.0);
            context.rotate(std::f64::consts::FRAC_PI_2);
        }
        180 | -180 => {
            context.translate(width, height);
            context.rotate(std::f64::consts::PI);
        }
        270 | -90 => {
            context.translate(0.0, width);
            context.rotate(-std::f64::consts::FRAC_PI_2);
        }
        _ => {}
    }

    context.translate(-crop_box.origin.x, -crop_box.origin.y);

    unsafe {
        CGContextDrawPDFPage(context.as_ptr() as *mut c_void, page);
    }

    let image = context
        .create_image()
        .ok_or_else(|| "Failed to create image from context".to_string())?;

    encode_png(&image)
}

fn encode_png(image: &core_graphics::image::CGImage) -> Result<Vec<u8>, String> {
    let data = unsafe { CFDataCreateMutable(ptr::null(), 0) };
    if data.is_null() {
        return Err("Failed to create mutable data".to_string());
    }

    let png_type = CFString::new("public.png");

    let dest = unsafe {
        CGImageDestinationCreateWithData(
            data,
            png_type.as_CFTypeRef() as *const c_void,
            1,
            ptr::null(),
        )
    };

    if dest.is_null() {
        unsafe { CFRelease(data as *const c_void) };
        return Err("Failed to create image destination".to_string());
    }

    unsafe {
        CGImageDestinationAddImage(dest, image.as_ptr() as *const c_void, ptr::null());
    }

    let success = unsafe { CGImageDestinationFinalize(dest) };

    unsafe { CFRelease(dest as *const c_void) };

    if !success {
        unsafe { CFRelease(data as *const c_void) };
        return Err("Failed to finalize PNG encoding".to_string());
    }

    let length = unsafe { CFDataGetLength(data as *const c_void) } as usize;
    let bytes_ptr = unsafe { CFDataGetBytePtr(data as *const c_void) };

    if bytes_ptr.is_null() || length == 0 {
        unsafe { CFRelease(data as *const c_void) };
        return Err("PNG data is empty".to_string());
    }

    let result = unsafe { std::slice::from_raw_parts(bytes_ptr, length).to_vec() };

    unsafe { CFRelease(data as *const c_void) };

    Ok(result)
}
