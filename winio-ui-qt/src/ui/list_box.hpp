#pragma once

#include "common.hpp"
#include <QAbstractItemView>
#include <QListWidget>
#include <QListWidgetItem>
#include <QWidget>
#include <memory>

STATIC_CAST_ASSERT(QListWidget, QWidget);

using QAbstractItemViewSelectionMode = QAbstractItemView::SelectionMode;

std::unique_ptr<QListWidget> new_list_widget(QWidget *parent);
void list_widget_connect_select(QListWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data);
