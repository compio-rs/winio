#include "group_box.hpp"

std::unique_ptr<QGroupBox> new_group_box(QWidget *parent) {
    return std::make_unique<QGroupBox>(parent);
}

void group_box_set_view(QGroupBox &w, QWidget *view) {
    auto layout = new QVBoxLayout(&w);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);
    layout->addWidget(view);
}

QSize group_box_title_size(QGroupBox const &w) {
    return w.fontMetrics().size(Qt::TextSingleLine, w.title());
}

QSize group_box_contents_size(QGroupBox const &w) {
    return w.contentsRect().size();
}
