#pragma once

#include <QString>
#include <QWidget>
#include <memory>
#include <rust/cxx.h>

bool is_dark();

rust::String widget_get_title(QWidget const &w);
void widget_set_title(QWidget &w, rust::Str s);

std::unique_ptr<QWidget> dummy_new_qwidget();
