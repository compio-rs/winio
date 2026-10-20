#pragma once

#include "../common.hpp"
#include <QFontMetrics>
#include <QGroupBox>
#include <QVBoxLayout>
#include <memory>

STATIC_CAST_ASSERT(QGroupBox, QWidget);

std::unique_ptr<QGroupBox> new_group_box(QWidget *parent);

void group_box_set_view(QGroupBox &w, QWidget *view);

QSize group_box_title_size(QGroupBox const &w);
QSize group_box_contents_size(QGroupBox const &w);
