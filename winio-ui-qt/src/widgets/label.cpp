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
