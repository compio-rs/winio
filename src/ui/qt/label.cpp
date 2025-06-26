#include "label.hpp"

std::unique_ptr<QLabel> new_label(QWidget *parent) {
    return std::make_unique<QLabel>(parent);
}
