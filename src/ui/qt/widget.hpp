#include <QCloseEvent>
#include <QMainWindow>
#include <QMoveEvent>
#include <QResizeEvent>
#include <QWidget>
#include <optional>
#include <rust/cxx.h>
#include <tuple>

using ::QWidget;
using c_void = void;

template <typename T> struct callback_traits;

template <typename Ret, typename... Args> struct callback_traits<Ret(Args...)> {
    using fn_type = rust::Fn<Ret(std::uint8_t const *, Args...)>;
    using type = std::optional<std::tuple<fn_type, std::uint8_t const *>>;
};

template <typename T> using callback_t = typename callback_traits<T>::type;
template <typename T>
using callback_fn_t = typename callback_traits<T>::fn_type;

struct WinioMainWindow : QMainWindow {
    callback_t<void(int, int)> m_resize_callback;
    callback_t<void(int, int)> m_move_callback;
    callback_t<bool()> m_close_callback;

    WinioMainWindow();

protected:
    void resizeEvent(QResizeEvent *event) override;
    void moveEvent(QMoveEvent *event) override;
    void closeEvent(QCloseEvent *event) override;
};

std::unique_ptr<QWidget> new_main_window();
void main_window_register_resize_event(QWidget &w,
                                       callback_fn_t<void(int, int)> callback,
                                       std::uint8_t const *data);
void main_window_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data);
void main_window_register_close_event(QWidget &w,
                                      callback_fn_t<bool()> callback,
                                      std::uint8_t const *data);

std::unique_ptr<QWidget> new_push_button(QWidget &parent);
void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data);