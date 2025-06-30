#pragma once

#include "common.hpp"
#include <QMessageBox>
#include <memory>

using QMessageBoxIcon = QMessageBox::Icon;
using QMessageBoxStandardButton = QMessageBox::StandardButton;

std::unique_ptr<QMessageBox> new_message_box(QWidget *parent);
void message_box_connect_finished(QMessageBox &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data);
QPushButton *message_box_add_button(QMessageBox &b, rust::Str text);
