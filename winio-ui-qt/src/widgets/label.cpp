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

QString label_font_family(const QLabel &w) { return w.font().family(); }

double label_font_size(const QLabel &w) { return w.font().pointSizeF(); }

bool label_font_bold(const QLabel &w) { return w.font().bold(); }

bool label_font_italic(const QLabel &w) { return w.font().italic(); }
