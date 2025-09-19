#include "window.hpp"

WinioMainWindow::WinioMainWindow(QWidget *parent)
    : QMainWindow(parent), m_resize_callback(std::nullopt),
      m_move_callback(std::nullopt), m_close_callback(std::nullopt) {
    setWindowFlags(windowFlags() | Qt::WindowMinMaxButtonsHint);
}

WinioMainWindow::~WinioMainWindow() {}

void WinioMainWindow::resizeEvent(QResizeEvent *event) {
    if (m_resize_callback) {
        auto &[callback, data] = *m_resize_callback;
        auto size = event->size();
        callback(data, size.width(), size.height());
    }
}

void WinioMainWindow::moveEvent(QMoveEvent *event) {
    if (m_move_callback) {
        auto &[callback, data] = *m_move_callback;
        auto pos = event->pos();
        callback(data, pos.x(), pos.y());
    }
}

void WinioMainWindow::closeEvent(QCloseEvent *event) {
    if (m_close_callback) {
        auto &[callback, data] = *m_close_callback;
        if (callback(data)) {
            event->ignore();
            return;
        }
    }
    event->accept();
}

std::unique_ptr<QMainWindow> new_main_window() {
    return std::make_unique<WinioMainWindow>(nullptr);
}

void main_window_register_resize_event(QMainWindow &w,
                                       callback_fn_t<void(int, int)> callback,
                                       std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_resize_callback =
        std::make_tuple(std::move(callback), data);
}

void main_window_register_move_event(QMainWindow &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_move_callback =
        std::make_tuple(std::move(callback), data);
}

void main_window_register_close_event(QMainWindow &w,
                                      callback_fn_t<bool()> callback,
                                      std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_close_callback =
        std::make_tuple(std::move(callback), data);
}
