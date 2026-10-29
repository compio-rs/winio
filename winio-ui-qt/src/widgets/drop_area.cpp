#include "drop_area.hpp"
#include <QDragEnterEvent>
#include <QDragMoveEvent>
#include <QDropEvent>

DropAreaWidget::DropAreaWidget(QWidget *parent,
                               callback_fn_t<void(const QMimeData &)> callback,
                               std::uint8_t const *data)
    : QWidget(parent), m_callback(callback), m_data(data) {
    setAcceptDrops(true);
}

void DropAreaWidget::dragEnterEvent(QDragEnterEvent *event) {
    event->acceptProposedAction();
}

void DropAreaWidget::dragMoveEvent(QDragMoveEvent *event) {
    event->acceptProposedAction();
}

void DropAreaWidget::dropEvent(QDropEvent *event) {
    event->acceptProposedAction();
    if (const QMimeData *data = event->mimeData()) {
        m_callback(m_data, *data);
    }
}

std::unique_ptr<DropAreaWidget>
new_drop_area(QWidget *parent, callback_fn_t<void(const QMimeData &)> callback,
              std::uint8_t const *data) {
    return std::make_unique<DropAreaWidget>(parent, callback, data);
}
