#include "webview.hpp"

std::unique_ptr<QWebEngineView> new_webview(QWebEngineProfile *profile,
                                            QWidget *parent) {
    return std::make_unique<QWebEngineView>(profile, parent);
}

void webview_connect_load_started(QWebEngineView &w,
                                  callback_fn_t<void()> callback,
                                  std::uint8_t const *data) {
    QObject::connect(&w, &QWebEngineView::loadStarted,
                     [callback, data]() { callback(data); });
}

void webview_connect_load_finished(QWebEngineView &w,
                                   callback_fn_t<void()> callback,
                                   std::uint8_t const *data) {
    QObject::connect(&w, &QWebEngineView::loadFinished,
                     [callback, data](bool) { callback(data); });
}

void webview_page_run_js(QWebEnginePage &page, QString const &js,
                         callback_fn_t<void(QString const &)> callback,
                         std::uint8_t const *data) {
    page.runJavaScript(js, [callback, data](const QVariant &result) {
        if (result.isNull()) {
            callback(data, QString());
        } else {
            callback(data, result.toString());
        }
    });
}

std::unique_ptr<QWebEngineProfile> new_webview_profile() {
    return std::make_unique<QWebEngineProfile>();
}

void webview_cookie_store_add(QWebEngineCookieStore &store, rust::Str cookies) {
    auto arr = QByteArray::fromRawData(cookies.data(), cookies.size());
    auto list = QNetworkCookie::parseCookies(arr);
    for (const auto &cookie : list) {
        store.setCookie(cookie);
    }
}

void webview_cookie_store_delete(QWebEngineCookieStore &store,
                                 rust::Str cookies) {
    auto arr = QByteArray::fromRawData(cookies.data(), cookies.size());
    auto list = QNetworkCookie::parseCookies(arr);
    for (const auto &cookie : list) {
        store.deleteCookie(cookie);
    }
}

void webview_cookie_store_connect_add(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data) {
    QObject::connect(&store, &QWebEngineCookieStore::cookieAdded,
                     [callback, data](const QNetworkCookie &cookie) {
                         callback(data, cookie);
                     });
}

void webview_cookie_store_connect_delete(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data) {
    QObject::connect(&store, &QWebEngineCookieStore::cookieRemoved,
                     [callback, data](const QNetworkCookie &cookie) {
                         callback(data, cookie);
                     });
}

QString cookie_to_raw(QNetworkCookie const &cookie) {
    return cookie.toRawForm(QNetworkCookie::Full);
}
