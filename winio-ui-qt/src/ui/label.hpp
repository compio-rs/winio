#pragma once

#include "common.hpp"
#include "edit.hpp"
#include <QLabel>

STATIC_CAST_ASSERT(QLabel, QWidget);

std::unique_ptr<QLabel> new_label(QWidget *parent);

void label_connect_link_activated(QLabel &w, callback_fn_t<void()> callback,
                                  std::uint8_t const *data);
