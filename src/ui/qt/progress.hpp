#pragma once

#include "common.hpp"
#include <QProgressBar>
#include <QWidget>
#include <memory>

STATIC_CAST_IMPL(QProgressBar, QWidget);

std::unique_ptr<QProgressBar> new_progress_bar(QWidget *parent);
