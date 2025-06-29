#pragma once

#include "common.hpp"
#include <QComboBox>
#include <QWidget>
#include <memory>

STATIC_CAST_ASSERT(QComboBox, QWidget);

std::unique_ptr<QComboBox> new_combo_box(QWidget *parent, bool editable);
void combo_box_connect_changed(QComboBox &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
void combo_box_connect_select(QComboBox &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data);

inline void combo_box_insert(QComboBox &w, int i, const QString &s) {
    w.insertItem(i, s);
}
