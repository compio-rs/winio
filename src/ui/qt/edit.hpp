#pragma once

#include "callback.hpp"
#include "common.hpp"
#include <QLineEdit>
#include <QTextEdit>
#include <QWidget>
#include <memory>

using QtAlignmentFlag = Qt::Alignment;
using QLineEditEchoMode = QLineEdit::EchoMode;

STATIC_CAST_IMPL(QLineEdit, QWidget);
STATIC_CAST_IMPL(QTextEdit, QWidget);

std::unique_ptr<QLineEdit> new_line_edit(QWidget *parent);
void line_edit_connect_changed(QLineEdit &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);

std::unique_ptr<QTextEdit> new_text_edit(QWidget *parent);
void text_edit_connect_changed(QTextEdit &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
