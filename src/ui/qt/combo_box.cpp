#include "combo_box.hpp"
#include <QComboBox>

std::unique_ptr<QWidget> new_combo_box(QWidget *parent, bool editable) {
    auto combo = std::make_unique<QComboBox>(parent);
    combo->setEditable(editable);
    return combo;
}

void combo_box_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data) {
    QObject::connect(static_cast<QComboBox *>(&w),
                     &QComboBox::currentTextChanged,
                     [callback, data](QString const &) { callback(data); });
}

void combo_box_connect_select(QWidget &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(static_cast<QComboBox *>(&w),
                     QOverload<int>::of(&QComboBox::currentIndexChanged),
                     [callback, data](int) { callback(data); });
}

rust::String combo_box_get_text(QWidget const &w) {
    auto text = static_cast<QComboBox const &>(w).currentText();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void combo_box_set_text(QWidget &w, rust::Str s) {
    static_cast<QComboBox &>(w).setCurrentText(
        QString::fromUtf8(s.data(), s.size()));
}

int combo_box_get_current_index(QWidget const &w) {
    return static_cast<QComboBox const &>(w).currentIndex();
}

void combo_box_set_current_index(QWidget &w, int i) {
    static_cast<QComboBox &>(w).setCurrentIndex(i);
}

void combo_box_insert(QWidget &w, int i, rust::Str s) {
    static_cast<QComboBox &>(w).insertItem(
        i, QString::fromUtf8(s.data(), s.size()));
}

void combo_box_remove(QWidget &w, int i) {
    static_cast<QComboBox &>(w).removeItem(i);
}

void combo_box_clear(QWidget &w) { static_cast<QComboBox &>(w).clear(); }

int combo_box_count(QWidget const &w) {
    return static_cast<QComboBox const &>(w).count();
}

rust::String combo_box_get(QWidget const &w, int i) {
    auto text = static_cast<QComboBox const &>(w).itemText(i);
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void combo_box_set(QWidget &w, int i, rust::Str s) {
    static_cast<QComboBox &>(w).setItemText(
        i, QString::fromUtf8(s.data(), s.size()));
}
