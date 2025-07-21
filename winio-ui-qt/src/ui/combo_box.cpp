#include "combo_box.hpp"
#include <QComboBox>

std::unique_ptr<QComboBox> new_combo_box(QWidget *parent) {
    return std::make_unique<QComboBox>(parent);
}

void combo_box_connect_changed(QComboBox &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(&w, &QComboBox::currentTextChanged,
                     [callback, data](QString const &) { callback(data); });
}

void combo_box_connect_select(QComboBox &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(&w, QOverload<int>::of(&QComboBox::currentIndexChanged),
                     [callback, data](int) { callback(data); });
}
