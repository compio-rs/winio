#include "widget.hpp"
#include <QApplication>

bool is_dark() {
    auto back = QApplication::palette().color(QPalette::Window);
    auto brightness =
        back.redF() * 0.299 + back.greenF() * 0.587 + back.blueF() * 0.114;
    return brightness < 0.5;
}

rust::String widget_get_title(const QWidget &w) {
    auto title = w.windowTitle();
    return rust::String{(const char16_t *)title.utf16(),
                        (std::size_t)title.size()};
}

void widget_set_title(QWidget &w, rust::Str s) {
    w.setWindowTitle(QString::fromUtf8(s.data(), s.size()));
}
