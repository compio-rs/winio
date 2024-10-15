#pragma once

#include "edit.hpp"

std::unique_ptr<QWidget> new_label(QWidget *parent);

rust::String label_get_text(QWidget const &w);
void label_set_text(QWidget &w, rust::Str s);

QtAlignmentFlag label_get_alignment(QWidget const &w);
void label_set_alignment(QWidget &w, QtAlignmentFlag flag);
