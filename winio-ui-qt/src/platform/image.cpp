#include "image.hpp"

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format) {
    return std::make_unique<QImage>(bits, width, height, stride, format);
}

std::unique_ptr<QImage> image_scaled(QImage const &image, int width,
                                     int height) {
    return std::make_unique<QImage>(
        image.scaled(width, height, Qt::IgnoreAspectRatio,
                     Qt::SmoothTransformation));
}
