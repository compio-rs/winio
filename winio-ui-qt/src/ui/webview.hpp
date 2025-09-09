#pragma once

#include "common.hpp"
#include <QWebEngineHistory>
#include <QWebEngineView>

STATIC_CAST_ASSERT(QWebEngineView, QWidget);

std::unique_ptr<QWebEngineView> new_webview(QWidget *parent);

void webview_connect_load_finished(QWebEngineView &w,
                                   callback_fn_t<void()> callback,
                                   std::uint8_t const *data);
