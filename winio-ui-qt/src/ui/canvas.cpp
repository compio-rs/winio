#include "canvas.hpp"
#include <QApplication>
#include <QBrush>
#include <QFont>
#include <QLinearGradient>
#include <QPen>
#include <QRadialGradient>

WinioCanvas::WinioCanvas(QWidget *parent)
    : QWidget(parent), m_paint_callback(std::nullopt),
      m_move_callback(std::nullopt), m_press_callback(std::nullopt),
      m_release_callback(std::nullopt), m_buffer() {
    setMouseTracking(true);
}

WinioCanvas::~WinioCanvas() {}

void WinioCanvas::paintEvent(QPaintEvent *) {
    QPainter painter(this);
    m_buffer.play(&painter);
}

void WinioCanvas::mouseMoveEvent(QMouseEvent *event) {
    if (m_move_callback) {
        auto &[callback, data] = *m_move_callback;
        auto pos = event->pos();
        callback(data, pos.x(), pos.y());
    }
}

void WinioCanvas::mousePressEvent(QMouseEvent *event) {
    if (m_press_callback) {
        auto &[callback, data] = *m_press_callback;
        callback(data, event->button());
    }
}

void WinioCanvas::mouseReleaseEvent(QMouseEvent *event) {
    if (m_release_callback) {
        auto &[callback, data] = *m_release_callback;
        callback(data, event->button());
    }
}

void WinioCanvas::wheelEvent(QWheelEvent *event) {
    if (m_wheel_callback) {
        auto &[callback, data] = *m_wheel_callback;
        auto delta = event->angleDelta();
        int sign = event->inverted() ? -1 : 1;
        callback(data, -delta.x() * sign, delta.y() * sign);
    }
}

std::unique_ptr<QWidget> new_canvas(QWidget *parent) {
    return std::make_unique<WinioCanvas>(parent);
}

void canvas_register_move_event(QWidget &w,
                                callback_fn_t<void(int, int)> callback,
                                std::uint8_t const *data) {
    static_cast<WinioCanvas &>(w).m_move_callback =
        std::make_tuple(std::move(callback), data);
}

void canvas_register_press_event(QWidget &w,
                                 callback_fn_t<void(QtMouseButton)> callback,
                                 std::uint8_t const *data) {
    static_cast<WinioCanvas &>(w).m_press_callback =
        std::make_tuple(std::move(callback), data);
}

void canvas_register_release_event(QWidget &w,
                                   callback_fn_t<void(QtMouseButton)> callback,
                                   std::uint8_t const *data) {
    static_cast<WinioCanvas &>(w).m_release_callback =
        std::make_tuple(std::move(callback), data);
}

void canvas_register_wheel_event(QWidget &w,
                                 callback_fn_t<void(int, int)> callback,
                                 std::uint8_t const *data) {
    static_cast<WinioCanvas &>(w).m_wheel_callback =
        std::make_tuple(std::move(callback), data);
}

std::unique_ptr<QPainter> canvas_new_painter(QWidget &w) {
    auto &c = static_cast<WinioCanvas &>(w);
    c.m_buffer = QPicture{};
    return std::make_unique<QPainter>(&c.m_buffer);
}

void painter_set_font(QPainter &p, rust::Str family, double size, bool italic,
                      bool bold) {
    auto font = QFont{QString::fromUtf8(family.data(), family.size()), -1,
                      bold ? QFont::Bold : QFont::Normal, italic};
    font.setPixelSize(std::max((int)size, 1));
    p.setFont(font);
}

QSizeF painter_measure_text(QPainter &p, QRectF rect, rust::Str text) {
    auto r = p.boundingRect(rect, QString::fromUtf8(text.data(), text.size()));
    return r.size();
}

void painter_draw_text(QPainter &p, QRectF rect, rust::Str text) {
    QTextOption option{};
    p.drawText(rect, QString::fromUtf8(text.data(), text.size()), option);
}

void color_transparent(QColor &c) { new (&c) QColor{Qt::transparent}; }

bool color_accent(QColor &c) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 6, 0)
    auto accent = QApplication::palette().color(QPalette::Accent);
    new (&c) QColor{accent};
    return true;
#else
    return false;
#endif
}

std::unique_ptr<QGradient> new_gradient_linear(QPointF start, QPointF end) {
    return std::make_unique<QLinearGradient>(start, end);
}

std::unique_ptr<QGradient> new_gradient_radial(QPointF center, double radius,
                                               QPointF origin) {
    return std::make_unique<QRadialGradient>(center, radius, origin);
}

void brush_set_transform(QBrush &b, double m11, double m12, double m21,
                         double m22, double m31, double m32) {
    b.setTransform(QTransform{m11, m12, m21, m22, m31, m32});
}

std::unique_ptr<QImage> new_image(int width, int height, int stride,
                                  const uchar *bits, QImage::Format format) {
    return std::make_unique<QImage>(bits, width, height, stride, format);
}

void painter_draw_image(QPainter &p, QRectF const &target, QImage const &image,
                        QRectF const &source) {
    p.drawImage(target, image, source);
}

std::unique_ptr<QPainterPath> new_path() {
    return std::make_unique<QPainterPath>();
}
