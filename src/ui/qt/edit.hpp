#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_line_edit(QWidget &parent);
void line_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
rust::String line_edit_get_text(QWidget const &w);
void line_edit_set_text(QWidget &w, rust::Str s);
