#pragma once

#include "../common.hpp"
#include <QImage>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QOpenGLWidget>
#include <QWidget>
#include <memory>

#include <winio-ui-qt/src/widgets/wgpu.rs.h>

using QtMouseButton = Qt::MouseButton;

struct WinioWgpuCanvas : QOpenGLWidget {
    callback_t<void(int, int)> m_move_callback;
    callback_t<void(QtMouseButton)> m_press_callback;
    callback_t<void(QtMouseButton)> m_release_callback;
    callback_t<void(int, int)> m_wheel_callback;
    callback_t<void(int)> m_key_down_callback;
    callback_t<void(int)> m_key_up_callback;
    callback_t<void(QString const &)> m_key_char_callback;
    QImage m_image;
    struct GlState;
    std::unique_ptr<GlState> m_gl;

    explicit WinioWgpuCanvas(QWidget *parent);
    ~WinioWgpuCanvas() override;

    void set_image(std::uint8_t const *data, int width, int height, int stride);
    void clear_image();

    bool set_dmabuf(int fd, int width, int height, int stride,
                    std::uint32_t fourcc);
    void clear_dmabuf();

protected:
    void paintEvent(QPaintEvent *event) override;
    void initializeGL() override;
    void paintGL() override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
    void wheelEvent(QWheelEvent *event) override;
    void keyPressEvent(QKeyEvent *event) override;
    void keyReleaseEvent(QKeyEvent *event) override;

private:
    void paint_image();
};

std::unique_ptr<QWidget> new_wgpu_canvas(QWidget *parent);
double wgpu_canvas_pixel_ratio(QWidget const &w);
void wgpu_canvas_set_image(QWidget const &w, std::uint8_t const *data,
                           std::uint32_t width, std::uint32_t height,
                           std::uint32_t stride);
void wgpu_canvas_clear_image(QWidget const &w);
void wgpu_canvas_clear_dmabuf(QWidget const &w);
bool wgpu_canvas_set_dmabuf(QWidget const &w, int fd, std::uint32_t width,
                            std::uint32_t height, std::uint32_t stride,
                            std::uint32_t fourcc);

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
void wgpu_canvas_register_key_down_event(QWidget &w,
                                         callback_fn_t<void(int)> callback,
                                         std::uint8_t const *data);
void wgpu_canvas_register_key_up_event(QWidget &w,
                                       callback_fn_t<void(int)> callback,
                                       std::uint8_t const *data);
void wgpu_canvas_register_key_char_event(
    QWidget &w, callback_fn_t<void(QString const &)> callback,
    std::uint8_t const *data);
