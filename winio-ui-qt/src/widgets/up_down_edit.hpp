#pragma once

#include "../common.hpp"
#include <QSpinBox>
#include <QWidget>
#include <memory>

STATIC_CAST_ASSERT(QSpinBox, QWidget);

std::unique_ptr<QSpinBox> new_spin_box(QWidget *parent);
void spin_box_connect_changed(QSpinBox &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data);
