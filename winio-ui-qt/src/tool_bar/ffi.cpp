#include "ffi.hpp"

std::unique_ptr<QToolBar> new_tool_bar(QWidget *window) {
    auto *main_window = static_cast<QMainWindow *>(window);
    auto toolbar = std::make_unique<QToolBar>(main_window);
    toolbar->setAllowedAreas(Qt::TopToolBarArea);
    toolbar->setMovable(false);
    toolbar->setFloatable(false);
    toolbar->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    main_window->addToolBar(Qt::TopToolBarArea, toolbar.get());
    return toolbar;
}

bool tool_bar_exists(QWidget *window) {
    auto *main_window = static_cast<QMainWindow *>(window);
    return main_window->findChild<QToolBar *>(QString(),
                                              Qt::FindDirectChildrenOnly) != nullptr;
}

QAction *tool_bar_add_action(QToolBar *toolbar) {
    return toolbar->addAction(QString());
}

QAction *tool_bar_add_separator(QToolBar *toolbar) {
    return toolbar->addSeparator();
}

QAction *tool_bar_add_menu_action(QToolBar *toolbar) {
    auto *action = new QAction(toolbar);
    action->setMenu(new QMenu(toolbar));
    toolbar->addAction(action);
    if (auto *button =
            qobject_cast<QToolButton *>(toolbar->widgetForAction(action))) {
        button->setPopupMode(QToolButton::InstantPopup);
    }
    return action;
}

QMenu *tool_bar_action_menu(QAction *action) { return action->menu(); }
