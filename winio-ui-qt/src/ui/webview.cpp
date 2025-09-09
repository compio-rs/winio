#include "webview.hpp"

std::unique_ptr<QWebEngineView> new_webview(QWidget *parent) {
    return std::make_unique<QWebEngineView>(parent);
}

void webview_connect_load_finished(QWebEngineView &w,
                                   callback_fn_t<void()> callback,
                                   std::uint8_t const *data) {
    QObject::connect(&w, &QWebEngineView::loadFinished,
                     [callback, data](bool) { callback(data); });
}
