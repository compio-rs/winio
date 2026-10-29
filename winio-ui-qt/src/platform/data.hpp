#pragma once

#include "../common.hpp"
#include <QImage>
#include <QMimeData>
#include <memory>
#include <rust/cxx.h>

std::unique_ptr<QMimeData> new_mime_data();
std::unique_ptr<QMimeData> clone_mime_data(const QMimeData &data);

rust::Vec<rust::String> mime_data_formats(const QMimeData &data);
rust::Vec<uint8_t> mime_data_get(const QMimeData &data, rust::Str mime);
QString mime_data_text(const QMimeData &data, rust::Str mime);
void mime_data_set(QMimeData &data, rust::Str mime,
                   rust::Slice<const uint8_t> bytes);
void mime_data_set_image(QMimeData &data, const QImage &image);
void mime_data_set_uris(QMimeData &data, rust::Slice<const QString> uris);
bool mime_data_has_image(const QMimeData &data);
rust::Vec<uint8_t> mime_data_image_png(const QMimeData &data);
std::unique_ptr<QImage> mime_data_image(const QMimeData &data);
rust::Vec<rust::String> mime_data_local_files(const QMimeData &data);
