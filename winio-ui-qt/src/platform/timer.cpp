#include "timer.hpp"

std::unique_ptr<QTimer> new_timer() { return std::make_unique<QTimer>(); }

void timer_connect_timeout(QTimer &timer, callback_fn_t<void()> callback,
                           std::uint8_t const *data) {
    QObject::connect(&timer, &QTimer::timeout,
                     [callback, data]() { callback(data); });
}
