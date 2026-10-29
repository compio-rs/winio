#include "clipboard.hpp"
#include "data.hpp"
#include <QClipboard>
#include <QGuiApplication>

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

void clipboard_clear() { QGuiApplication::clipboard()->clear(); }
