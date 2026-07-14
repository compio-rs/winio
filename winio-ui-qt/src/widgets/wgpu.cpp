#include "wgpu.hpp"
#include <QGuiApplication>
#include <QMouseEvent>
#include <QWheelEvent>

#include <qpa/qplatformnativeinterface.h>

WinioWgpuCanvas::WinioWgpuCanvas(QWidget *parent)
    : QWidget(parent), m_move_callback(std::nullopt),
      m_press_callback(std::nullopt), m_release_callback(std::nullopt),
      m_wheel_callback(std::nullopt) {
    setMouseTracking(true);
    QCoreApplication::setAttribute(Qt::AA_DontCreateNativeWidgetSiblings);
    setAttribute(Qt::WA_DontCreateNativeAncestors);
    setAttribute(Qt::WA_NativeWindow);
    setAttribute(Qt::WA_PaintOnScreen);
}

WinioWgpuCanvas::~WinioWgpuCanvas() {}

QPaintEngine *WinioWgpuCanvas::paintEngine() const { return nullptr; }

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

std::unique_ptr<QWidget> new_wgpu_canvas(QWidget *parent) {
    return std::make_unique<WinioWgpuCanvas>(parent);
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

WaylandDescriptor wgpu_canvas_wayland_descriptor(QWidget const &w) {
    auto interface = QGuiApplication::platformNativeInterface();
    auto display = static_cast<wl_display *>(
        interface->nativeResourceForIntegration("wl_display"));
    auto surface = static_cast<wl_surface *>(
        interface->nativeResourceForWindow("surface", w.windowHandle()));
    return {display, surface};
}

XcbDescriptor wgpu_canvas_xcb_descriptor(QWidget const &w) {
    auto interface = QGuiApplication::platformNativeInterface();
    auto connection = static_cast<xcb_connection_t *>(
        interface->nativeResourceForIntegration("connection"));
    auto screen =
        (std::int32_t)(std::intptr_t)interface->nativeResourceForIntegration(
            "x11screen");
    auto window = (std::uint32_t)w.winId();
    return {connection, screen, window};
}
