#pragma once

#include "callback.hpp"
#include <QCloseEvent>
#include <QMainWindow>
#include <QMoveEvent>
#include <QResizeEvent>
#include <QWidget>
#include <memory>

struct WinioMainWindow : public QMainWindow {
    callback_t<void(int, int)> m_resize_callback;
    callback_t<void(int, int)> m_move_callback;
    callback_t<bool()> m_close_callback;

    WinioMainWindow(QWidget *parent);

protected:
    void resizeEvent(QResizeEvent *event) override;
    void moveEvent(QMoveEvent *event) override;
    void closeEvent(QCloseEvent *event) override;
};

std::unique_ptr<QWidget> new_main_window(QWidget *parent);
void main_window_register_resize_event(QWidget &w,
                                       callback_fn_t<void(int, int)> callback,
                                       std::uint8_t const *data);
void main_window_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data);
void main_window_register_close_event(QWidget &w,
                                      callback_fn_t<bool()> callback,
                                      std::uint8_t const *data);
