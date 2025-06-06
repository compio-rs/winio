#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

using QtAlignmentFlag = Qt::AlignmentFlag;

std::unique_ptr<QWidget> new_line_edit(QWidget *parent);
void line_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
rust::String line_edit_get_text(QWidget const &w);
void line_edit_set_text(QWidget &w, rust::Str s);

QtAlignmentFlag line_edit_get_alignment(QWidget const &w);
void line_edit_set_alignment(QWidget &w, QtAlignmentFlag flag);

bool line_edit_is_password(QWidget const &w);
void line_edit_set_password(QWidget &w, bool v);

std::unique_ptr<QWidget> new_text_edit(QWidget *parent);
void text_edit_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);

rust::String text_edit_get_text(QWidget const &w);
void text_edit_set_text(QWidget &w, rust::Str s);

QtAlignmentFlag text_edit_get_alignment(QWidget const &w);
void text_edit_set_alignment(QWidget &w, QtAlignmentFlag flag);
