#include "edit.hpp"
#include <QLineEdit>
#include <QTextEdit>

std::unique_ptr<QWidget> new_line_edit(QWidget *parent) {
    return std::make_unique<QLineEdit>(parent);
}

void line_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(static_cast<QLineEdit *>(&w), &QLineEdit::textEdited,
                     [callback, data](QString const &) { callback(data); });
}

rust::String line_edit_get_text(QWidget const &w) {
    auto text = static_cast<QLineEdit const &>(w).text();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void line_edit_set_text(QWidget &w, rust::Str s) {
    static_cast<QLineEdit &>(w).setText(QString::fromUtf8(s.data(), s.size()));
}

QtAlignmentFlag line_edit_get_alignment(QWidget const &w) {
    return (QtAlignmentFlag)(int)static_cast<QLineEdit const &>(w).alignment();
}

void line_edit_set_alignment(QWidget &w, QtAlignmentFlag flag) {
    static_cast<QLineEdit &>(w).setAlignment(flag);
}

bool line_edit_is_password(QWidget const &w) {
    return static_cast<QLineEdit const &>(w).echoMode() != QLineEdit::Normal;
}

void line_edit_set_password(QWidget &w, bool v) {
    static_cast<QLineEdit &>(w).setEchoMode(v ? QLineEdit::Password
                                              : QLineEdit::Normal);
}

std::unique_ptr<QWidget> new_text_edit(QWidget *parent) {
    return std::make_unique<QTextEdit>(parent);
}

void text_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(static_cast<QTextEdit *>(&w), &QTextEdit::textChanged,
                     [callback, data]() { callback(data); });
}

rust::String text_edit_get_text(QWidget const &w) {
    auto text = static_cast<QTextEdit const &>(w).toPlainText();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void text_edit_set_text(QWidget &w, rust::Str s) {
    static_cast<QTextEdit &>(w).setText(QString::fromUtf8(s.data(), s.size()));
}

QtAlignmentFlag text_edit_get_alignment(QWidget const &w) {
    return (QtAlignmentFlag)(int)static_cast<QTextEdit const &>(w).alignment();
}

void text_edit_set_alignment(QWidget &w, QtAlignmentFlag flag) {
    static_cast<QTextEdit &>(w).setAlignment(flag);
}
