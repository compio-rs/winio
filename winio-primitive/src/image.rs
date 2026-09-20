//! Image data helpers.

use bytemuck::Pod;

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
