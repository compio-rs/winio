#pragma once

#include "callback.hpp"
#include "common.hpp"
#include <QAbstractButton>
#include <QCheckBox>
#include <QPushButton>
#include <QRadioButton>
#include <QWidget>
#include <memory>

using QtCheckState = Qt::CheckState;

STATIC_CAST_IMPL(QAbstractButton, QWidget);
STATIC_CAST_IMPL(QPushButton, QAbstractButton);
STATIC_CAST_IMPL(QCheckBox, QAbstractButton);
STATIC_CAST_IMPL(QRadioButton, QAbstractButton);

std::unique_ptr<QPushButton> new_push_button(QWidget *parent);
std::unique_ptr<QCheckBox> new_check_box(QWidget *parent);
std::unique_ptr<QRadioButton> new_radio_button(QWidget *parent);

void push_button_connect_clicked(QAbstractButton &w,
                                 callback_fn_t<void()> callback,
                                 std::uint8_t const *data);
