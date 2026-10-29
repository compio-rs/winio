#pragma once

#include "../common.hpp"
#include <QImage>
#include <QMimeData>
#include <QObject>
#include <memory>

class ClipboardContext : public QObject {
public:
    explicit ClipboardContext(QObject *parent = nullptr) : QObject(parent) {}
};

std::unique_ptr<ClipboardContext>
clipboard_connect_changed(callback_fn_t<void()> callback,
                          std::uint8_t const *data);

std::unique_ptr<QMimeData> clipboard_read();
void clipboard_write(const QMimeData &data);
void clipboard_write_text(const QString &text);
void clipboard_write_rtf(const QString &rtf);
void clipboard_write_html(const QString &html);
void clipboard_write_image(const QImage &image);
void clipboard_write_uris(rust::Slice<QString> uris);
