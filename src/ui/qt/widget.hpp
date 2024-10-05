#include <QWidget>
#include <rust/cxx.h>

using ::QWidget;
using c_void = void;

std::unique_ptr<QWidget> new_main_window();
void main_window_close_event(QWidget &window,
                             rust::Fn<bool(std::uint8_t const *)> callback,
                             std::uint8_t const *data);

std::unique_ptr<QWidget> new_push_button(QWidget &parent);
