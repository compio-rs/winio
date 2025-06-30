#pragma once

#include "common.hpp"
#include <QFileDialog>
#include <QWidget>
#include <memory>

using QFileDialogAcceptMode = QFileDialog::AcceptMode;
using QFileDialogFileMode = QFileDialog::FileMode;

std::unique_ptr<QFileDialog> new_file_dialog(QWidget *parent);
void file_dialog_connect_finished(QFileDialog &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data);
rust::Vec<rust::String> file_dialog_files(QFileDialog const &b);
