#include "scroll_bar.hpp"

std::unique_ptr<QScrollBar> new_scroll_bar(QWidget *parent) {
    auto bar = std::make_unique<QScrollBar>(parent);
    bar->setTracking(true);
    bar->setOrientation(Qt::Horizontal);
    return bar;
}

std::unique_ptr<QSlider> new_slider(QWidget *parent) {
    auto bar = std::make_unique<QSlider>(parent);
    bar->setTracking(true);
    bar->setOrientation(Qt::Horizontal);
    bar->setTickPosition(QSlider::TicksBothSides);
    bar->setPageStep(0);
    return bar;
}

void scroll_bar_connect_moved(QAbstractSlider &w,
                              callback_fn_t<void(QAbstractSlider &)> callback,
                              std::uint8_t const *data) {
    QObject::connect(&w, &QAbstractSlider::valueChanged,
                     [&w, callback, data](int) { callback(data, w); });
}
