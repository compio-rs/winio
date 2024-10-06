#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_push_button(QWidget &parent);
void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data);
