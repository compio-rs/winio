#pragma once

#include "common.hpp"
#include <QScrollArea>

using QtScrollBarPolicy = Qt::ScrollBarPolicy;

STATIC_CAST_ASSERT(QScrollArea, QWidget);

inline std::unique_ptr<QScrollArea> new_scroll_area(QWidget *parent) {
    return std::make_unique<QScrollArea>(parent);
}
