#include "menu_button.hpp"
#include <QIcon>
#include <QPixmap>

std::unique_ptr<QToolButton> new_menu_button(QWidget *parent) {
    auto button = std::make_unique<QToolButton>(parent);
    button->setPopupMode(QToolButton::InstantPopup);
    button->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    button->setMenu(new QMenu(button.get()));
    return button;
}

QMenu *menu_button_menu(QToolButton *button) { return button->menu(); }

void menu_button_set_icon(QToolButton &button, QImage const &icon) {
    button.setIcon(QIcon(QPixmap::fromImage(icon)));
}

void menu_button_clear_icon(QToolButton &button) { button.setIcon(QIcon()); }
