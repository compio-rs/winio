#include "edit.hpp"
#include <QLineEdit>

std::unique_ptr<QWidget> new_line_edit(QWidget *parent, bool password) {
    auto edit = std::make_unique<QLineEdit>(parent);
    if (password) {
        edit->setEchoMode(QLineEdit::Password);
    }
    return edit;
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
