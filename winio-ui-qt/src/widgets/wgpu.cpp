#include "wgpu.hpp"

#include <QMouseEvent>
#include <QPainter>
#include <QWheelEvent>

WinioWgpuCanvas::WinioWgpuCanvas(QWidget *parent)
    : QWidget(parent), m_move_callback(std::nullopt),
      m_press_callback(std::nullopt), m_release_callback(std::nullopt),
      m_wheel_callback(std::nullopt), m_key_down_callback(std::nullopt),
      m_key_up_callback(std::nullopt), m_key_char_callback(std::nullopt) {
    setMouseTracking(true);
    setFocusPolicy(Qt::StrongFocus);
    // The whole area is covered by the frame.
    setAttribute(Qt::WA_OpaquePaintEvent);
}

WinioWgpuCanvas::~WinioWgpuCanvas() {}

void WinioWgpuCanvas::paintEvent(QPaintEvent *) {
    QPainter painter(this);
    if (m_image.isNull()) {
        painter.fillRect(rect(), palette().window());
        return;
    }
    painter.drawImage(rect(), m_image);
}

void WinioWgpuCanvas::mouseMoveEvent(QMouseEvent *event) {
    if (m_move_callback) {
        auto &[callback, data] = *m_move_callback;
        auto pos = event->pos();
        callback(data, pos.x(), pos.y());
    }
}

void WinioWgpuCanvas::mousePressEvent(QMouseEvent *event) {
    if (m_press_callback) {
        auto &[callback, data] = *m_press_callback;
        callback(data, event->button());
    }
}

void WinioWgpuCanvas::mouseReleaseEvent(QMouseEvent *event) {
    if (m_release_callback) {
        auto &[callback, data] = *m_release_callback;
        callback(data, event->button());
    }
}

void WinioWgpuCanvas::wheelEvent(QWheelEvent *event) {
    if (m_wheel_callback) {
        auto &[callback, data] = *m_wheel_callback;
        auto delta = event->angleDelta();
        int sign = event->inverted() ? -1 : 1;
        callback(data, -delta.x() * sign, delta.y() * sign);
    }
}

void WinioWgpuCanvas::keyPressEvent(QKeyEvent *event) {
    if (m_key_down_callback) {
        auto &[callback, data] = *m_key_down_callback;
        callback(data, event->key());
    }
    if (m_key_char_callback) {
        auto &[callback, data] = *m_key_char_callback;
        callback(data, event->text());
    }
}

void WinioWgpuCanvas::keyReleaseEvent(QKeyEvent *event) {
    if (m_key_up_callback) {
        auto &[callback, data] = *m_key_up_callback;
        callback(data, event->key());
    }
}

std::unique_ptr<QWidget> new_wgpu_canvas(QWidget *parent) {
    return std::make_unique<WinioWgpuCanvas>(parent);
}

double wgpu_canvas_pixel_ratio(QWidget const &w) {
    return w.devicePixelRatioF();
}

void wgpu_canvas_set_image(QWidget const &w, std::uint8_t const *data,
                           std::uint32_t width, std::uint32_t height,
                           std::uint32_t stride) {
    auto &c =
        const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w));
    // The pixels stay mapped until the canvas clears the image again, so
    // `QImage` can wrap them without a copy.
    c.m_image =
        QImage(data, int(width), int(height), int(stride), QImage::Format_ARGB32);
    c.update();
}

void wgpu_canvas_clear_image(QWidget const &w) {
    auto &c =
        const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w));
    c.m_image = QImage();
}

void wgpu_canvas_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_move_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_press_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_press_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_release_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_release_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_wheel_event(QWidget &w,
                                      callback_fn_t<void(int, int)> callback,
                                      std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_wheel_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_down_event(QWidget &w,
                                         callback_fn_t<void(int)> callback,
                                         std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_down_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_up_event(QWidget &w,
                                       callback_fn_t<void(int)> callback,
                                       std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_up_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_char_event(
    QWidget &w, callback_fn_t<void(QString const &)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_char_callback =
        std::make_tuple(std::move(callback), data);
}
