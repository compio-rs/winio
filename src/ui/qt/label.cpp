#include "label.hpp"

std::unique_ptr<QLabel> new_label(QWidget *parent) {
    return std::make_unique<QLabel>(parent);
}

QtAlignmentFlag label_get_alignment(QLabel const &w) {
    return (QtAlignmentFlag)(int)w.alignment();
}

void label_set_alignment(QLabel &w, QtAlignmentFlag flag) {
    w.setAlignment(flag);
}
