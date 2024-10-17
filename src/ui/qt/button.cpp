#include "button.hpp"

#include <QCheckBox>
#include <QPushButton>
#include <QRadioButton>

std::unique_ptr<QWidget> new_push_button(QWidget *parent) {
    return std::make_unique<QPushButton>(parent);
}

std::unique_ptr<QWidget> new_check_box(QWidget *parent) {
    return std::make_unique<QCheckBox>(parent);
}

std::unique_ptr<QWidget> new_radio_button(QWidget *parent) {
    auto button = std::make_unique<QRadioButton>(parent);
    button->setAutoExclusive(false);
    return button;
}

void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data) {
    QObject::connect(static_cast<QAbstractButton *>(&w),
                     &QAbstractButton::clicked,
                     [callback, data](bool) { callback(data); });
}

rust::String push_button_get_text(QWidget const &w) {
    auto text = static_cast<QAbstractButton const &>(w).text();
    return rust::String{(const char16_t *)text.utf16(),
                        (std::size_t)text.size()};
}

void push_button_set_text(QWidget &w, rust::Str s) {
    static_cast<QAbstractButton &>(w).setText(
        QString::fromUtf8(s.data(), s.size()));
}

bool check_box_is_checked(QWidget const &w) {
    return static_cast<QCheckBox const &>(w).checkState() != Qt::Unchecked;
}

void check_box_set_checked(QWidget &w, bool v) {
    static_cast<QCheckBox &>(w).setCheckState(v ? Qt::Checked : Qt::Unchecked);
}

bool radio_button_is_checked(QWidget const &w) {
    return static_cast<QRadioButton const &>(w).isChecked();
}

void radio_button_set_checked(QWidget &w, bool v) {
    static_cast<QRadioButton &>(w).setChecked(v);
}
