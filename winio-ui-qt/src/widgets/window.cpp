#include "window.hpp"
#include <QTimer>

WinioMainWindow::WinioMainWindow(QWidget *parent)
    : QMainWindow(parent), m_move_callback(std::nullopt),
      m_close_callback(std::nullopt), m_theme_callback(std::nullopt) {
    setWindowFlags(windowFlags() | Qt::WindowMinMaxButtonsHint);
}

WinioMainWindow::~WinioMainWindow() {}

bool WinioMainWindow::event(QEvent *event) {
    const auto type = event->type();
#if QT_VERSION >= QT_VERSION_CHECK(6, 6, 0)
    const bool dpi_changed = type == QEvent::DevicePixelRatioChange;
#else
    const bool dpi_changed = type == QEvent::ScreenChangeInternal;
#endif
    const bool handled = QMainWindow::event(event);
    if (dpi_changed) {
        // The DPI event is delivered recursively. Queue the notification so
        // every child has seen the new DPI before the component re-renders.
        QTimer::singleShot(0, this, [this]() {
            auto *content = static_cast<WinioCentralWidget *>(centralWidget());
            if (content) {
                content->notify_resize();
            }
        });
    }
    return handled;
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

void WinioMainWindow::changeEvent(QEvent *event) {
    auto type = event->type();
    if (type == QEvent::ThemeChange || type == QEvent::PaletteChange ||
        type == QEvent::StyleChange) {
        if (m_theme_callback) {
            auto &[callback, data] = *m_theme_callback;
            callback(data);
        }
    }
}

WinioCentralWidget::WinioCentralWidget(QWidget *parent)
    : QWidget(parent), m_resize_callback(std::nullopt) {}

WinioCentralWidget::~WinioCentralWidget() {}

void WinioCentralWidget::notify_resize() {
    if (m_resize_callback) {
        auto &[callback, data] = *m_resize_callback;
        callback(data, width(), height());
    }
}

void WinioCentralWidget::resizeEvent(QResizeEvent *event) {
    QWidget::resizeEvent(event);
    notify_resize();
}

std::unique_ptr<QMainWindow> new_main_window() {
    return std::make_unique<WinioMainWindow>(nullptr);
}

std::unique_ptr<QWidget> new_central_widget(QMainWindow &window) {
    auto content = std::make_unique<WinioCentralWidget>(&window);
    window.setCentralWidget(content.get());
    return content;
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

void main_window_register_theme_event(QMainWindow &w,
                                      callback_fn_t<void()> callback,
                                      std::uint8_t const *data) {
    static_cast<WinioMainWindow &>(w).m_theme_callback =
        std::make_tuple(std::move(callback), data);
}

void central_widget_register_resize_event(QWidget &w,
                                          callback_fn_t<void(int, int)> callback,
                                          std::uint8_t const *data) {
    static_cast<WinioCentralWidget &>(w).m_resize_callback =
        std::make_tuple(std::move(callback), data);
}
