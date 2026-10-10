use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::path::Path;

pub fn data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}

#[cfg(not(windows))]
pub fn extract_icon_png(_path: &Path) -> Option<Vec<u8>> {
    None
}

#[cfg(windows)]
pub fn extract_icon_png(path: &Path) -> Option<Vec<u8>> {
    use std::{mem::size_of, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::{
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject,
            BITMAPINFO, BI_RGB, DIB_RGB_COLORS,
        },
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL},
        },
    };

    const SIZE: usize = 32;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut info = SHFILEINFOW::default();
    let found = unsafe {
        SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &mut info,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if found == 0 || info.hIcon.is_null() {
        return None;
    }

    let result = (|| {
        let mut bitmap = BITMAPINFO::default();
        bitmap.bmiHeader.biSize = size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        bitmap.bmiHeader.biWidth = SIZE as i32;
        bitmap.bmiHeader.biHeight = -(SIZE as i32);
        bitmap.bmiHeader.biPlanes = 1;
        bitmap.bmiHeader.biBitCount = 32;
        bitmap.bmiHeader.biCompression = BI_RGB;
        let mut bits = ptr::null_mut();
        let dib = unsafe {
            CreateDIBSection(
                ptr::null_mut(),
                &bitmap,
                DIB_RGB_COLORS,
                &mut bits,
                ptr::null_mut(),
                0,
            )
        };
        if dib.is_null() || bits.is_null() {
            return None;
        }
        let dc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if dc.is_null() {
            unsafe { DeleteObject(dib) };
            return None;
        }
        let old = unsafe { SelectObject(dc, dib) };
        let image = if old.is_null() || old as isize == -1 {
            None
        } else {
            unsafe { ptr::write_bytes(bits as *mut u8, 0, SIZE * SIZE * 4) };
            let drawn = unsafe {
                DrawIconEx(dc, 0, 0, info.hIcon, SIZE as i32, SIZE as i32, 0, ptr::null_mut(), DI_NORMAL)
            };
            if drawn == 0 {
                None
            } else {
                let mut bgra = unsafe { std::slice::from_raw_parts(bits as *const u8, SIZE * SIZE * 4) }.to_vec();
                if bgra.chunks_exact(4).all(|pixel| pixel[3] == 0) {
                    // Legacy icons carry transparency in a mask, not the alpha channel.
                    // Drawing on white as well as black recovers that mask without opaque squares.
                    unsafe { ptr::write_bytes(bits as *mut u8, 255, SIZE * SIZE * 4) };
                    if unsafe { DrawIconEx(dc, 0, 0, info.hIcon, SIZE as i32, SIZE as i32, 0, ptr::null_mut(), DI_NORMAL) } == 0 {
                        unsafe { SelectObject(dc, old); DeleteDC(dc); DeleteObject(dib); }
                        return None;
                    }
                    let white = unsafe { std::slice::from_raw_parts(bits as *const u8, SIZE * SIZE * 4) };
                    for (black, white) in bgra.chunks_exact_mut(4).zip(white.chunks_exact(4)) {
                        let difference = (0..3).map(|channel| white[channel].saturating_sub(black[channel])).max().unwrap_or(255);
                        black[3] = 255 - difference;
                    }
                }
                let mut rgba = Vec::with_capacity(bgra.len());
                for pixel in bgra.chunks_exact(4) {
                    let alpha = pixel[3];
                    let color = |channel: u8| if alpha == 0 { 0 } else { ((channel as u32 * 255) / alpha as u32).min(255) as u8 };
                    rgba.extend_from_slice(&[color(pixel[2]), color(pixel[1]), color(pixel[0]), alpha]);
                }
                let mut encoded = Vec::new();
                let written = (|| {
                    let mut encoder = png::Encoder::new(&mut encoded, SIZE as u32, SIZE as u32);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder.write_header().ok()?.write_image_data(&rgba).ok()?;
                    Some(())
                })();
                written.map(|_| encoded)
            }
        };
        if !old.is_null() && old as isize != -1 {
            unsafe { SelectObject(dc, old) };
        }
        unsafe {
            DeleteDC(dc);
            DeleteObject(dib);
        }
        image
    })();
    unsafe { DestroyIcon(info.hIcon) };
    result
}
