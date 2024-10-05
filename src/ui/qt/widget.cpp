#include "widget.hpp"

#include <QCloseEvent>
#include <QMainWindow>
#include <QPushButton>
#include <optional>
#include <tuple>

struct WinioMainWindow : QMainWindow {
    std::optional<
        std::tuple<rust::Fn<bool(std::uint8_t const *)>, std::uint8_t const *>>
        m_close_callback;

    WinioMainWindow() : QMainWindow(), m_close_callback(std::nullopt) {}

    void closeEvent(QCloseEvent *event) override {
        if (m_close_callback.has_value()) {
            auto &[callback, data] = m_close_callback.value();
            if (callback(data)) {
                event->ignore();
            }
        }
    }
};

std::unique_ptr<QWidget> new_main_window() {
    return std::make_unique<QMainWindow>();
}

void main_window_close_event(QWidget &window,
                             rust::Fn<bool(std::uint8_t const *)> callback,
                             std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(window).m_close_callback =
        std::make_tuple(std::move(callback), data);
}

std::unique_ptr<QWidget> new_push_button(QWidget &parent) {
    return std::make_unique<QPushButton>(&parent);
}
