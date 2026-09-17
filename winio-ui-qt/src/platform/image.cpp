#include "image.hpp"

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format) {
    return std::make_unique<QImage>(bits, width, height, stride, format);
}

std::unique_ptr<QImage> image_copy(const QImage &image) {
    return std::make_unique<QImage>(image.copy());
}
