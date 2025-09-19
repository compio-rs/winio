#pragma once

#include "common.hpp"
#include <QPointer>
#include <QString>
#include <QWidget>
#include <rust/cxx.h>

bool is_dark();

std::unique_ptr<QWidget> new_widget(QWidget *parent);

using QWidgetPointer = QPointer<QWidget>;

namespace rust {
template <> struct IsRelocatable<QWidgetPointer> : std::true_type {};
} // namespace rust

static_assert(sizeof(QWidgetPointer) == 2 * sizeof(std::size_t));

QWidgetPointer widget_weak(QWidget *w);
