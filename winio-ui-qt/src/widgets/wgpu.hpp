#pragma once

#include "../common.hpp"
#include <QWidget>
#include <memory>

struct wl_display;
struct wl_surface;
struct xcb_connection_t;

#include <winio-ui-qt/src/widgets/wgpu.rs.h>

using QtMouseButton = Qt::MouseButton;

struct WinioWgpuCanvas : public QWidget {
    callback_t<void(int, int)> m_move_callback;
    callback_t<void(QtMouseButton)> m_press_callback;
    callback_t<void(QtMouseButton)> m_release_callback;
    callback_t<void(int, int)> m_wheel_callback;

    WinioWgpuCanvas(QWidget *parent);
    ~WinioWgpuCanvas() override;

    QPaintEngine *paintEngine() const override;

protected:
    void mouseMoveEvent(QMouseEvent *event) override;
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
    void wheelEvent(QWheelEvent *event) override;
};

std::unique_ptr<QWidget> new_wgpu_canvas(QWidget *parent);
void wgpu_canvas_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data);
void wgpu_canvas_register_press_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data);
void wgpu_canvas_register_release_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data);
void wgpu_canvas_register_wheel_event(QWidget &w,
                                      callback_fn_t<void(int, int)> callback,
                                      std::uint8_t const *data);

WaylandDescriptor wgpu_canvas_wayland_descriptor(QWidget const &w);
XcbDescriptor wgpu_canvas_xcb_descriptor(QWidget const &w);
