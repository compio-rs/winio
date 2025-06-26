#include "progress.hpp"

std::unique_ptr<QProgressBar> new_progress_bar(QWidget *parent) {
    return std::make_unique<QProgressBar>(parent);
}
