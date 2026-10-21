#pragma once

#include "../common.hpp"
#include <QAction>
#include <QMainWindow>
#include <QMenu>
#include <QToolBar>
#include <QToolButton>
#include <QWidget>
#include <memory>

std::unique_ptr<QToolBar> new_tool_bar(QWidget *window);

bool tool_bar_exists(QWidget *window);

QAction *tool_bar_add_action(QToolBar *toolbar);

QAction *tool_bar_add_separator(QToolBar *toolbar);

QAction *tool_bar_add_menu_action(QToolBar *toolbar);

QMenu *tool_bar_action_menu(QAction *action);

STATIC_CAST_ASSERT(QToolBar, QWidget);
