#pragma once

#include "../common.hpp"
#include <QMimeData>
#include <QWidget>
#include <memory>

class DropAreaWidget : public QWidget {
  public:
    DropAreaWidget(QWidget *parent,
                   callback_fn_t<void(const QMimeData &)> callback,
                   std::uint8_t const *data);

  protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dragMoveEvent(QDragMoveEvent *event) override;
    void dropEvent(QDropEvent *event) override;

  private:
    callback_fn_t<void(const QMimeData &)> m_callback;
    std::uint8_t const *m_data;
};

STATIC_CAST_ASSERT(DropAreaWidget, QWidget);

std::unique_ptr<DropAreaWidget>
new_drop_area(QWidget *parent, callback_fn_t<void(const QMimeData &)> callback,
              std::uint8_t const *data);
