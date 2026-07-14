#include "edit.hpp"

std::unique_ptr<QLineEdit> new_line_edit(QWidget *parent) {
    return std::make_unique<QLineEdit>(parent);
}

void line_edit_connect_changed(QLineEdit &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(&w, &QLineEdit::textEdited,
                     [callback, data](QString const &) { callback(data); });
}

std::unique_ptr<QTextEdit> new_text_edit(QWidget *parent) {
    return std::make_unique<QTextEdit>(parent);
}

void text_edit_connect_changed(QTextEdit &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(&w, &QTextEdit::textChanged,
                     [callback, data]() { callback(data); });
}
