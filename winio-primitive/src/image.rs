//! Image data helpers.

use bytemuck::Pod;

/// Premultiply the alpha channel of RGBA float pixels.
///
/// The length of `pixels` must be a multiple of 4.
pub fn premultiply_rgba_f32(pixels: &mut [f32]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3];
        if a == 0.0 {
            pixel[0] = 0.0;
            pixel[1] = 0.0;
            pixel[2] = 0.0;
        } else if a != 1.0 {
            pixel[0] *= a;
            pixel[1] *= a;
            pixel[2] *= a;
        }
    }
}

/// Unpremultiply the alpha channel of RGBA float pixels.
///
/// The length of `pixels` must be a multiple of 4.
pub fn unpremultiply_rgba_f32(pixels: &mut [f32]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3];
        if a != 0.0 && a != 1.0 {
            pixel[0] /= a;
            pixel[1] /= a;
            pixel[2] /= a;
        }
    }
}

/// Premultiply the alpha channel of RGBA pixels.
///
/// The length of `pixels` must be a multiple of 4.
pub fn premultiply_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3] as u16;
        if a == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        } else if a != 255 {
            pixel[0] = ((pixel[0] as u16 * a + 127) / 255) as u8;
            pixel[1] = ((pixel[1] as u16 * a + 127) / 255) as u8;
            pixel[2] = ((pixel[2] as u16 * a + 127) / 255) as u8;
        }
    }
}

/// Unpremultiply the alpha channel of RGBA pixels.
///
/// The length of `pixels` must be a multiple of 4.
pub fn unpremultiply_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let a = pixel[3] as u16;
        if a != 0 && a != 255 {
            pixel[0] = ((pixel[0] as u16 * 255 + a / 2) / a).min(255) as u8;
            pixel[1] = ((pixel[1] as u16 * 255 + a / 2) / a).min(255) as u8;
            pixel[2] = ((pixel[2] as u16 * 255 + a / 2) / a).min(255) as u8;
        }
    }
}

/// Convert straight RGBA pixels to premultiplied BGRA in place.
///
/// The length of `pixels` must be a multiple of 4.
pub fn to_premultiplied_bgra8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let a = a as u16;
        pixel[0] = ((b as u16 * a + 127) / 255) as u8;
        pixel[1] = ((g as u16 * a + 127) / 255) as u8;
        pixel[2] = ((r as u16 * a + 127) / 255) as u8;
    }
}

/// Convert premultiplied BGRA pixels to straight RGBA in place.
///
/// The length of `pixels` must be a multiple of 4.
pub fn to_straight_rgba8(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let [b, g, r, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let alpha = a as u16;
        let (r, g, b) = if a == 0 {
            (0, 0, 0)
        } else {
            (
                ((r as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((g as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
                ((b as u16 * 255 + alpha / 2) / alpha).min(255) as u8,
            )
        };
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
    }
}

/// Copy the first `row` bytes of each `stride`-sized row of `data` into a
/// tightly packed `Vec<T>`.
pub fn packed_rows<T: Pod>(data: &[u8], stride: usize, row: usize, height: usize) -> Vec<T> {
    if stride == row {
        bytemuck::allocation::pod_collect_to_vec(&data[..row * height])
    } else {
        let mut packed = Vec::with_capacity(row * height);
        for line in data.chunks_exact(stride).take(height) {
            packed.extend_from_slice(&line[..row]);
        }
        bytemuck::allocation::pod_collect_to_vec(&packed)
    }
}
