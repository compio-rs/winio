#include "button.hpp"

std::unique_ptr<QPushButton> new_push_button(QWidget *parent) {
    return std::make_unique<QPushButton>(parent);
}

std::unique_ptr<QCheckBox> new_check_box(QWidget *parent) {
    return std::make_unique<QCheckBox>(parent);
}

std::unique_ptr<QRadioButton> new_radio_button(QWidget *parent) {
    auto button = std::make_unique<QRadioButton>(parent);
    button->setAutoExclusive(false);
    return button;
}

void push_button_connect_clicked(QAbstractButton &w,
                                 callback_fn_t<void()> callback,
                                 std::uint8_t const *data) {
    QObject::connect(&w, &QAbstractButton::clicked,
                     [callback, data](bool) { callback(data); });
}
