#pragma once

#include "../common.hpp"
#include <QAbstractButton>
#include <QImage>
#include <QMenu>
#include <QToolButton>
#include <QWidget>
#include <memory>

STATIC_CAST_ASSERT(QToolButton, QAbstractButton);
STATIC_CAST_ASSERT(QToolButton, QWidget);

std::unique_ptr<QToolButton> new_menu_button(QWidget *parent);

QMenu *menu_button_menu(QToolButton *button);

void menu_button_set_icon(QToolButton &button, QImage const &icon);

void menu_button_clear_icon(QToolButton &button);
