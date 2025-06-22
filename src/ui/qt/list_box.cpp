#include "list_box.hpp"
#include <QAbstractItemView>
#include <QListWidget>

std::unique_ptr<QWidget> new_list_widget(QWidget *parent) {
    auto list = std::make_unique<QListWidget>(parent);
    list->setSelectionMode(QAbstractItemView::MultiSelection);
    return list;
}

void list_widget_connect_select(QWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data) {
    QObject::connect(static_cast<QListWidget *>(&w),
                     &QListWidget::itemSelectionChanged,
                     [callback, data]() { callback(data); });
}

bool list_widget_is_selected(QWidget const &w, int i) {
    return static_cast<QListWidget const &>(w).item(i)->isSelected();
}

void list_widget_set_selected(QWidget &w, int i, bool v) {
    static_cast<QListWidget &>(w).item(i)->setSelected(v);
}

void list_widget_insert(QWidget &w, int i, rust::Str s) {
    static_cast<QListWidget &>(w).insertItem(
        i, QString::fromUtf8(s.data(), s.size()));
}

void list_widget_remove(QWidget &w, int i) {
    auto &list = static_cast<QListWidget &>(w);
    list.removeItemWidget(list.item(i));
}

void list_widget_clear(QWidget &w) { static_cast<QListWidget &>(w).clear(); }

int list_widget_count(QWidget const &w) {
    return static_cast<QListWidget const &>(w).count();
}

rust::String list_widget_get(QWidget const &w, int i) {
    auto text = static_cast<QListWidget const &>(w).item(i)->text();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void list_widget_set(QWidget &w, int i, rust::Str s) {
    static_cast<QListWidget &>(w).item(i)->setText(
        QString::fromUtf8(s.data(), s.size()));
}
