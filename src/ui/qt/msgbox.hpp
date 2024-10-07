#pragma once

#include "callback.hpp"
#include <QMessageBox>
#include <memory>

using QMessageBoxIcon = QMessageBox::Icon;
using QMessageBoxStandardButton = QMessageBox::StandardButton;

std::unique_ptr<QMessageBox> new_message_box();
std::unique_ptr<QMessageBox> new_message_box(QWidget &parent);
void message_box_connect_finished(QMessageBox &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data);
void message_box_set_texts(QMessageBox &b, rust::Str title, rust::Str msg,
                           rust::Str instr);
QPushButton *message_box_add_button(QMessageBox &b, rust::Str text);
