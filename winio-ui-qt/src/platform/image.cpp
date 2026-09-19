#include "image.hpp"

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format) {
    return std::make_unique<QImage>(bits, width, height, stride, format);
}

std::unique_ptr<QImage> image_copy(const QImage &image) {
    return std::make_unique<QImage>(image.copy());
}

std::unique_ptr<QImage> image_to_rgba8(const QImage &image) {
    return std::make_unique<QImage>(
        image.convertToFormat(QImage::Format_RGBA8888));
}

std::size_t image_bytes_per_line(const QImage &image) {
    return image.bytesPerLine();
}

rust::Slice<const uint8_t> image_bytes(const QImage &image) {
    return rust::Slice<const uint8_t>(
        reinterpret_cast<const uint8_t *>(image.constBits()),
        static_cast<size_t>(image.sizeInBytes()));
}
