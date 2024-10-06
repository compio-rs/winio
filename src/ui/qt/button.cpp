#include "button.hpp"

#include <QPushButton>

std::unique_ptr<QWidget> new_push_button(QWidget &parent) {
    return std::make_unique<QPushButton>(&parent);
}

void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data) {
    QObject::connect(static_cast<QPushButton *>(&w), &QPushButton::clicked,
                     [callback, data](bool) { callback(data); });
}
