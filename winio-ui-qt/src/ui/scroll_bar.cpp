#include "scroll_bar.hpp"

std::unique_ptr<QScrollBar> new_scroll_bar(QWidget *parent) {
    auto bar = std::make_unique<QScrollBar>(parent);
    bar->setTracking(true);
    return bar;
}

void scroll_bar_connect_moved(QScrollBar &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(&w, &QScrollBar::valueChanged,
                     [callback, data](int) { callback(data); });
}
