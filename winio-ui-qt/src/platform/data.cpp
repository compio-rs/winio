#include "data.hpp"
#include <QByteArray>
#include <QUrl>

namespace {

rust::String to_string(const QString &s) {
    const QByteArray bytes = s.toUtf8();
    return rust::String(bytes.constData(), bytes.size());
}

} // namespace

std::unique_ptr<QMimeData> new_mime_data() {
    return std::make_unique<QMimeData>();
}

std::unique_ptr<QMimeData> clone_mime_data(const QMimeData &data) {
    auto copy = std::make_unique<QMimeData>();
    for (const QString &format : data.formats()) {
        if (format == "application/x-qt-image") {
            copy->setImageData(data.imageData());
        } else if (format == "application/x-color") {
            copy->setColorData(data.colorData());
        } else {
            copy->setData(format, data.data(format));
        }
    }
    return copy;
}

rust::Vec<rust::String> mime_data_formats(const QMimeData &data) {
    rust::Vec<rust::String> result;
    for (const QString &format : data.formats()) {
        result.push_back(to_string(format));
    }
    return result;
}

QByteArray mime_data_get(const QMimeData &data, rust::Str mime) {
    return data.data(
        QString::fromUtf8(mime.data(), static_cast<qsizetype>(mime.size())));
}

QString mime_data_text(const QMimeData &data, rust::Str mime) {
    const QString type =
        QString::fromUtf8(mime.data(), static_cast<qsizetype>(mime.size()));
    if (type == "text/plain" || type == "text/plain;charset=utf-8") {
        return data.text();
    }
    if (type == "text/html") {
        return data.html();
    }
    return QString::fromUtf8(data.data(type));
}

void mime_data_set(QMimeData &data, rust::Str mime,
                   rust::Slice<const uint8_t> bytes) {
    const QString type =
        QString::fromUtf8(mime.data(), static_cast<qsizetype>(mime.size()));
    data.setData(type, QByteArray(reinterpret_cast<const char *>(bytes.data()),
                                  static_cast<qsizetype>(bytes.size())));
}

void mime_data_set_image(QMimeData &data, const QImage &image) {
    data.setImageData(image);
}

void mime_data_set_uris(QMimeData &data, rust::Slice<const QString> uris) {
    QList<QUrl> urls;
    for (std::size_t i = 0; i < uris.size(); ++i) {
        urls.push_back(QUrl::fromLocalFile(uris[i]));
    }
    data.setUrls(urls);
}

bool mime_data_has_image(const QMimeData &data) { return data.hasImage(); }

std::unique_ptr<QImage> mime_data_image(const QMimeData &data) {
    if (data.hasImage()) {
        QImage image = qvariant_cast<QImage>(data.imageData());
        if (!image.isNull()) {
            return std::make_unique<QImage>(std::move(image));
        }
    }
    for (const QString &format : data.formats()) {
        if (!format.startsWith("image/")) {
            continue;
        }
        const QByteArray name = format.toUtf8();
        QImage image = QImage::fromData(data.data(format), name.constData());
        if (!image.isNull()) {
            return std::make_unique<QImage>(std::move(image));
        }
    }
    return nullptr;
}

rust::Vec<rust::String> mime_data_local_files(const QMimeData &data) {
    rust::Vec<rust::String> result;
    for (const QUrl &url : data.urls()) {
        if (!url.isLocalFile()) {
            continue;
        }
        const QString path = url.toLocalFile();
        if (!path.isEmpty()) {
            result.push_back(to_string(path));
        }
    }
    return result;
}
