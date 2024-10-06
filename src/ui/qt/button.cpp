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

rust::String push_button_get_text(QWidget const &w) {
    auto text = static_cast<QPushButton const &>(w).text();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void push_button_set_text(QWidget &w, rust::Str s) {
    static_cast<QPushButton &>(w).setText(
        QString::fromUtf8(s.data(), s.size()));
}
