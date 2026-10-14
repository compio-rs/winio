#include "ffi.hpp"

QMenuBar *new_menu_bar(QWidget *window) {
    auto *main_window = static_cast<QMainWindow *>(window);
    return main_window->menuBar();
}

bool menu_bar_exists(QWidget *window) {
    auto *main_window = static_cast<QMainWindow *>(window);
    return main_window->findChild<QMenuBar *>(QString(),
                                              Qt::FindDirectChildrenOnly) != nullptr;
}

QMenu *menu_bar_add_menu(QMenuBar *menu_bar) {
    return menu_bar->addMenu(QString());
}

QMenu *menu_add_menu(QMenu *menu) { return menu->addMenu(QString()); }

QAction *menu_add_action(QMenu *menu) { return menu->addAction(QString()); }

std::unique_ptr<QActionGroup> new_action_group(QMenu *parent) {
    return std::make_unique<QActionGroup>(parent);
}

QAction *new_action(QMenu *parent) { return new QAction(parent); }

void action_connect_triggered(QAction &action, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(&action, &QAction::triggered,
                     [callback, data](bool) { callback(data); });
}
