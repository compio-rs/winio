#include "up_down_edit.hpp"

std::unique_ptr<QSpinBox> new_spin_box(QWidget *parent) {
    return std::make_unique<QSpinBox>(parent);
}

void spin_box_connect_changed(QSpinBox &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(&w, qOverload<int>(&QSpinBox::valueChanged),
                     [callback, data](int) { callback(data); });
}
