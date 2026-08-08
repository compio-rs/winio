#include "label.hpp"

std::unique_ptr<QLabel> new_label(QWidget *parent) {
    return std::make_unique<QLabel>(parent);
}

void label_connect_link_activated(QLabel &w, callback_fn_t<void()> callback,
                                  std::uint8_t const *data) {
    QObject::connect(&w, &QLabel::linkActivated,
                     [callback, data](const QString &href) {
                         if (href.isEmpty()) {
                             callback(data);
                         }
                     });
}

void label_set_font(QLabel &w, rust::Str family, double size, bool bold,
                    bool italic) {
    auto font = w.font();
    font.setFamily(QString::fromUtf8(family.data(), family.size()));
    font.setPointSizeF(size);
    font.setBold(bold);
    font.setItalic(italic);
    w.setFont(font);
}
