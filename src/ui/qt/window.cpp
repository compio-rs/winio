#include "window.hpp"

WinioMainWindow::WinioMainWindow()
    : QMainWindow(), m_close_callback(std::nullopt) {}

void WinioMainWindow::resizeEvent(QResizeEvent *event) {
    if (m_resize_callback.has_value()) {
        auto &[callback, data] = m_resize_callback.value();
        auto size = event->size();
        callback(data, size.width(), size.height());
    }
}

void WinioMainWindow::moveEvent(QMoveEvent *event) {
    if (m_move_callback.has_value()) {
        auto &[callback, data] = m_move_callback.value();
        auto pos = event->pos();
        callback(data, pos.x(), pos.y());
    }
}

void WinioMainWindow::closeEvent(QCloseEvent *event) {
    if (m_close_callback.has_value()) {
        auto &[callback, data] = m_close_callback.value();
        if (callback(data)) {
            event->ignore();
        }
    }
}

std::unique_ptr<QWidget> new_main_window() {
    return std::make_unique<QMainWindow>();
}

void main_window_register_resize_event(QWidget &w,
                                       callback_fn_t<void(int, int)> callback,
                                       std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_resize_callback =
        std::make_tuple(std::move(callback), data);
}

void main_window_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_move_callback =
        std::make_tuple(std::move(callback), data);
}

void main_window_register_close_event(QWidget &w,
                                      callback_fn_t<bool()> callback,
                                      std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_close_callback =
        std::make_tuple(std::move(callback), data);
}
