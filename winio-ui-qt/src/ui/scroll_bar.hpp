#pragma once

#include "common.hpp"
#include <QAbstractSlider>
#include <QScrollBar>
#include <QSlider>
#include <QWidget>
#include <memory>

using QtOrientation = Qt::Orientation;

STATIC_CAST_ASSERT(QAbstractSlider, QWidget);
STATIC_CAST_ASSERT(QScrollBar, QAbstractSlider);
STATIC_CAST_ASSERT(QSlider, QAbstractSlider);

std::unique_ptr<QScrollBar> new_scroll_bar(QWidget *parent);
std::unique_ptr<QSlider> new_slider(QWidget *parent);

void scroll_bar_connect_moved(QAbstractSlider &w,
                              callback_fn_t<void(QAbstractSlider &)> callback,
                              std::uint8_t const *data);
