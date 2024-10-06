#pragma once

#include <QString>
#include <QWidget>
#include <rust/cxx.h>

bool is_dark();

rust::String widget_get_title(QWidget const &w);
void widget_set_title(QWidget &w, rust::Str s);
