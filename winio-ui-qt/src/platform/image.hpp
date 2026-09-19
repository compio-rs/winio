#pragma once

#include <QImage>
#include <memory>
#include <rust/cxx.h>

using QImageFormat = QImage::Format;

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format);
std::unique_ptr<QImage> image_copy(const QImage &image);
std::unique_ptr<QImage> image_to_rgba8(const QImage &image);
std::size_t image_bytes_per_line(const QImage &image);
rust::Slice<const uint8_t> image_bytes(const QImage &image);
