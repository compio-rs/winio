#include "label.hpp"
#include <QLabel>

std::unique_ptr<QWidget> new_label(QWidget *parent) {
    return std::make_unique<QLabel>(parent);
}

rust::String label_get_text(const QWidget &w) {
    auto text = static_cast<QLabel const &>(w).text();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void label_set_text(QWidget &w, rust::Str s) {
    static_cast<QLabel &>(w).setText(QString::fromUtf8(s.data(), s.size()));
}

QtAlignmentFlag label_get_alignment(QWidget const &w) {
    return (QtAlignmentFlag)(int)static_cast<QLabel const &>(w).alignment();
}

void label_set_alignment(QWidget &w, QtAlignmentFlag flag) {
    static_cast<QLabel &>(w).setAlignment(flag);
}
