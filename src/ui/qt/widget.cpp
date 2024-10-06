#include "widget.hpp"

#include <QPushButton>

bool is_dark() {
    if (qApp) {
        auto back = qApp->palette().color(QPalette::Window);
        auto brightness =
            back.redF() * 0.299 + back.greenF() * 0.587 + back.blueF() * 0.114;
        if (brightness > 0.5) {
            return true;
        }
    }
    return false;
}

rust::String widget_get_title(const QWidget &w) {
    auto title = w.windowTitle();
    return rust::String{(const char16_t *)title.utf16(),
                        (std::size_t)title.size()};
}

void widget_set_title(QWidget &w, rust::Str s) {
    w.setWindowTitle(QString::fromUtf8(s.data(), s.size()));
}

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

std::unique_ptr<QWidget> new_push_button(QWidget &parent) {
    return std::make_unique<QPushButton>(&parent);
}

void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data) {
    QObject::connect(static_cast<QPushButton *>(&w), &QPushButton::clicked,
                     [callback, data](bool) { callback(data); });
}
