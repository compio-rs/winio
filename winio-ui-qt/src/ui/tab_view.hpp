#pragma once

#include "common.hpp"
#include <QTabWidget>

STATIC_CAST_ASSERT(QTabWidget, QWidget);

std::unique_ptr<QTabWidget> new_tab_widget(QWidget *parent);

void tab_widget_connect_changed(QTabWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data);
