#pragma once

#include "common.hpp"
#include "edit.hpp"
#include <QLabel>

std::unique_ptr<QLabel> new_label(QWidget *parent);

STATIC_CAST_IMPL(QLabel, QWidget);

QtAlignmentFlag label_get_alignment(QLabel const &w);
void label_set_alignment(QLabel &w, QtAlignmentFlag flag);
