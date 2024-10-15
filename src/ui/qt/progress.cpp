#include "progress.hpp"
#include <QProgressBar>

std::unique_ptr<QWidget> new_progress_bar(QWidget *parent) {
    return std::make_unique<QProgressBar>(parent);
}

void progress_bar_set_range(QWidget &w, int min, int max) {
    static_cast<QProgressBar &>(w).setRange(min, max);
}

int progress_bar_get_minimum(const QWidget &w) {
    return static_cast<QProgressBar const &>(w).minimum();
}

int progress_bar_get_maximum(const QWidget &w) {
    return static_cast<QProgressBar const &>(w).maximum();
}

void progress_bar_set_value(QWidget &w, int v) {
    static_cast<QProgressBar &>(w).setValue(v);
}

int progress_bar_get_value(const QWidget &w) {
    return static_cast<QProgressBar const &>(w).value();
}
