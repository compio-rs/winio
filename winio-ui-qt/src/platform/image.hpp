#pragma once

#include <QImage>
#include <memory>

using QImageFormat = QImage::Format;

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format);

std::unique_ptr<QImage> image_scaled(QImage const &image, int width,
                                     int height);
