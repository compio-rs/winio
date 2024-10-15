#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

using QtAlignmentFlag = Qt::AlignmentFlag;

std::unique_ptr<QWidget> new_line_edit(QWidget *parent, bool password);
void line_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
rust::String line_edit_get_text(QWidget const &w);
void line_edit_set_text(QWidget &w, rust::Str s);

QtAlignmentFlag line_edit_get_alignment(QWidget const &w);
void line_edit_set_alignment(QWidget &w, QtAlignmentFlag flag);
