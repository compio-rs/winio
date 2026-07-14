#include "tab_view.hpp"

std::unique_ptr<QTabWidget> new_tab_widget(QWidget *parent) {
    auto widget = std::make_unique<QTabWidget>(parent);
    widget->setTabsClosable(false);
    return widget;
}

void tab_widget_connect_changed(QTabWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data) {
    QObject::connect(&w, &QTabWidget::currentChanged,
                     [callback, data](int) { callback(data); });
}
