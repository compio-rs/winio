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
