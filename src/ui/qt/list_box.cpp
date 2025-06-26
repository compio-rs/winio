#include "list_box.hpp"

std::unique_ptr<QListWidget> new_list_widget(QWidget *parent) {
    auto list = std::make_unique<QListWidget>(parent);
    list->setSelectionMode(QAbstractItemView::MultiSelection);
    return list;
}

void list_widget_connect_select(QListWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data) {
    QObject::connect(&w, &QListWidget::itemSelectionChanged,
                     [callback, data]() { callback(data); });
}
