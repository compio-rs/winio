#pragma once

#include "common.hpp"
#include <QString>
#include <QWidget>
#include <rust/cxx.h>

bool is_dark();

std::unique_ptr<QWidget> new_widget(QWidget *parent);
