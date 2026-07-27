#pragma once

#include "../common.hpp"
#include <QTimer>
#include <memory>

std::unique_ptr<QTimer> new_timer();

void timer_connect_timeout(QTimer &timer, callback_fn_t<void()> callback,
                           std::uint8_t const *data);
