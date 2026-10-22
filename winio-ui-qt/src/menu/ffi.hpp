#pragma once

#include "../common.hpp"
#include <QAction>
#include <QActionGroup>
#include <QImage>
#include <QMainWindow>
#include <QMenu>
#include <QMenuBar>
#include <QPointer>
#include <QWidget>
#include <memory>
#include <utility>

class MenuConnection {
public:
    explicit MenuConnection(QMetaObject::Connection connection)
        : connection(std::move(connection)) {}
    ~MenuConnection() { QObject::disconnect(connection); }

    MenuConnection(const MenuConnection &) = delete;
    MenuConnection &operator=(const MenuConnection &) = delete;

private:
    QMetaObject::Connection connection;
};

using QActionPointer = QPointer<QAction>;

namespace rust {
template <> struct IsRelocatable<QActionPointer> : std::true_type {};
} // namespace rust

static_assert(sizeof(QActionPointer) == 2 * sizeof(std::size_t));

QActionPointer action_weak(QAction *action);

QMenuBar *new_menu_bar(QWidget *window);

bool menu_bar_exists(QWidget *window);

QMenu *menu_bar_add_menu(QMenuBar *menu_bar);

QMenu *menu_add_menu(QMenu *menu);

QAction *menu_add_action(QMenu *menu);

std::unique_ptr<QActionGroup> new_action_group(QMenu *parent);

QAction *new_action(QMenu *parent);

void action_set_exclusive(QAction &action);

void action_set_icon(QAction &action, QImage const &icon);

void action_clear_icon(QAction &action);

void action_connect_triggered(QAction &action, callback_fn_t<void()> callback,
                              std::uint8_t const *data);

void action_remove_from_parent(QAction *action);
