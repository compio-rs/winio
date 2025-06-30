#pragma once

#include "common.hpp"
#include "edit.hpp"
#include <QLabel>

inline std::unique_ptr<QLabel> new_label(QWidget *parent) {
    return std::make_unique<QLabel>(parent);
}

STATIC_CAST_ASSERT(QLabel, QWidget);
