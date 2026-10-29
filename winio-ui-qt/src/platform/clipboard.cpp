#include "clipboard.hpp"
#include "data.hpp"
#include <QByteArray>
#include <QClipboard>
#include <QGuiApplication>
#include <QUrl>

std::unique_ptr<ClipboardContext>
clipboard_connect_changed(callback_fn_t<void()> callback,
                          std::uint8_t const *data) {
    auto context = std::make_unique<ClipboardContext>();
    QObject::connect(QGuiApplication::clipboard(), &QClipboard::dataChanged,
                     context.get(), [callback, data]() { callback(data); });
    return context;
}

std::unique_ptr<QMimeData> clipboard_read() {
    const QMimeData *data = QGuiApplication::clipboard()->mimeData();
    if (!data || data->formats().isEmpty()) {
        return nullptr;
    }
    return clone_mime_data(*data);
}

void clipboard_write(const QMimeData &data) {
    QGuiApplication::clipboard()->setMimeData(clone_mime_data(data).release());
}

void clipboard_write_text(const QString &text) {
    QGuiApplication::clipboard()->setText(text);
}

void clipboard_write_rtf(const QString &rtf) {
    const QByteArray bytes = rtf.toUtf8();
    auto *data = new QMimeData();
    data->setData("text/rtf", bytes);
    data->setData("application/rtf", bytes);
    QGuiApplication::clipboard()->setMimeData(data);
}

void clipboard_write_html(const QString &html) {
    auto *data = new QMimeData();
    data->setHtml(html);
    QGuiApplication::clipboard()->setMimeData(data);
}

void clipboard_write_image(const QImage &image) {
    QGuiApplication::clipboard()->setImage(image);
}

void clipboard_write_uris(rust::Slice<QString> uris) {
    QList<QUrl> urls;
    for (std::size_t i = 0; i < uris.size(); ++i) {
        urls.push_back(QUrl::fromLocalFile(uris[i]));
    }
    auto *data = new QMimeData();
    data->setUrls(urls);
    QGuiApplication::clipboard()->setMimeData(data);
}
