#pragma once

#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_progress_bar(QWidget *parent);

void progress_bar_set_range(QWidget &w, int min, int max);
int progress_bar_get_minimum(QWidget const &w);
int progress_bar_get_maximum(QWidget const &w);

void progress_bar_set_value(QWidget &w, int v);
int progress_bar_get_value(QWidget const &w);
