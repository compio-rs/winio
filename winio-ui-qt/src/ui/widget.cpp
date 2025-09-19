#include "widget.hpp"
#include <QApplication>

bool is_dark() {
    auto back = QApplication::palette().color(QPalette::Window);
    auto brightness =
        back.redF() * 0.299 + back.greenF() * 0.587 + back.blueF() * 0.114;
    return brightness < 0.5;
}

std::unique_ptr<QWidget> new_widget(QWidget *parent) {
    return std::make_unique<QWidget>(parent);
}
