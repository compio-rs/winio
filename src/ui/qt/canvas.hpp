#pragma once

#include "callback.hpp"
#include <QGradient>
#include <QImage>
#include <QMouseEvent>
#include <QPaintEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPicture>
#include <QWidget>
#include <memory>

using QtMouseButton = Qt::MouseButton;
using QtSizeMode = Qt::SizeMode;
using QImageFormat = QImage::Format;

struct WinioCanvas : public QWidget {
    callback_t<void()> m_paint_callback;
    callback_t<void(int, int)> m_move_callback;
    callback_t<void(QtMouseButton)> m_press_callback;
    callback_t<void(QtMouseButton)> m_release_callback;

    QPicture m_buffer;

    WinioCanvas(QWidget *parent);
    ~WinioCanvas() override;

protected:
    void paintEvent(QPaintEvent *event) override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void mousePressEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
};

std::unique_ptr<QWidget> new_canvas(QWidget *parent);
void canvas_register_move_event(QWidget &w,
                                callback_fn_t<void(int, int)> callback,
                                std::uint8_t const *data);
void canvas_register_press_event(QWidget &w,
                                 callback_fn_t<void(QtMouseButton)> callback,
                                 std::uint8_t const *data);
void canvas_register_release_event(QWidget &w,
                                   callback_fn_t<void(QtMouseButton)> callback,
                                   std::uint8_t const *data);

std::unique_ptr<QPainter> canvas_new_painter(QWidget &w);
void painter_set_font(QPainter &p, rust::Str family, double size, bool italic,
                      bool bold);
QSizeF painter_measure_text(QPainter &p, QRectF rect, rust::Str text);
void painter_draw_text(QPainter &p, QRectF rect, rust::Str text);

void color_transparent(QColor &c);
std::unique_ptr<QBrush> new_brush(QColor const &c);
std::unique_ptr<QPen> new_pen(QBrush const &b, double width);

std::unique_ptr<QGradient> new_gradient_linear(QPointF start, QPointF end);
std::unique_ptr<QGradient> new_gradient_radial(QPointF center, double radius,
                                               QPointF origin);

std::unique_ptr<QBrush> new_brush_gradient(QGradient const &g);
void brush_set_transform(QBrush &b, double m11, double m12, double m21,
                         double m22, double m31, double m32);

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format);
void painter_draw_image(QPainter &p, QRectF const &target, QImage const &image,
                        QRectF const &source);

std::unique_ptr<QPainterPath> new_path();
