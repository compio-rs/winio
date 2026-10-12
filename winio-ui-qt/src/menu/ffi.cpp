#include "ffi.hpp"

QMenuBar *new_menu_bar(QWidget *window) {
    auto *main_window = static_cast<QMainWindow *>(window);
    return main_window->menuBar();
}

QMenu *menu_bar_add_menu(QMenuBar *menu_bar) {
    return menu_bar->addMenu(QString());
}

QMenu *menu_add_menu(QMenu *menu) { return menu->addMenu(QString()); }

QAction *menu_add_action(QMenu *menu) { return menu->addAction(QString()); }

void action_set_exclusive(QAction &action) {
    auto *group = new QActionGroup(&action);
    group->setExclusive(true);
    group->addAction(&action);
}

void action_connect_triggered(QAction &action, callback_fn_t<void()> callback,
                              std::uint8_t const *data) {
    QObject::connect(&action, &QAction::triggered,
                     [callback, data](bool) { callback(data); });
}
