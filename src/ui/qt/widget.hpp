#pragma once

#include "callback.hpp"
#include <QCloseEvent>
#include <QMainWindow>
#include <QMoveEvent>
#include <QResizeEvent>
#include <QString>
#include <QWidget>

bool is_dark();

rust::String widget_get_title(QWidget const &w);
void widget_set_title(QWidget &w, rust::Str s);

struct WinioMainWindow : QMainWindow {
    callback_t<void(int, int)> m_resize_callback;
    callback_t<void(int, int)> m_move_callback;
    callback_t<bool()> m_close_callback;

    WinioMainWindow();

protected:
    void resizeEvent(QResizeEvent *event) override;
    void moveEvent(QMoveEvent *event) override;
    void closeEvent(QCloseEvent *event) override;
};

std::unique_ptr<QWidget> new_main_window();
void main_window_register_resize_event(QWidget &w,
                                       callback_fn_t<void(int, int)> callback,
                                       std::uint8_t const *data);
void main_window_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data);
void main_window_register_close_event(QWidget &w,
                                      callback_fn_t<bool()> callback,
                                      std::uint8_t const *data);

std::unique_ptr<QWidget> new_push_button(QWidget &parent);
void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data);
