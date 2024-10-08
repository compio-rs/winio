#include "canvas.hpp"
#include <QBrush>
#include <QFont>
#include <QPen>

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

std::unique_ptr<QPainter> canvas_new_painter(QWidget &w) {
    auto &c = static_cast<WinioCanvas &>(w);
    c.m_buffer = QPicture{};
    return std::make_unique<QPainter>(&c.m_buffer);
}

void painter_set_font(QPainter &p, rust::Str family, double size, bool italic,
                      bool bold) {
    auto font = QFont{QString::fromUtf8(family.data(), family.size()), -1,
                      bold ? QFont::Bold : QFont::Normal, italic};
    font.setPixelSize((int)size);
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

QColor color_transparent() { return QColor{Qt::transparent}; }

std::unique_ptr<QBrush> new_brush(QColor c) {
    return std::make_unique<QBrush>(c);
}

std::unique_ptr<QPen> new_pen(QBrush const &b, double width) {
    return std::make_unique<QPen>(b, width);
}
