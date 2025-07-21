#pragma once

#include "common.hpp"
#include <QScrollBar>
#include <QWidget>
#include <memory>

using QtOrientation = Qt::Orientation;

STATIC_CAST_ASSERT(QScrollBar, QWidget);

std::unique_ptr<QScrollBar> new_scroll_bar(QWidget *parent);

void scroll_bar_connect_moved(QScrollBar &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data);
