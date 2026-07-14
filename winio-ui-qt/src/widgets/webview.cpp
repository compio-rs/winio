#include "webview.hpp"

std::unique_ptr<QWebEngineView> new_webview(QWebEngineProfile *profile,
                                            QWidget *parent) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 4, 0)
    return std::make_unique<QWebEngineView>(profile, parent);
#else
    auto webview = std::make_unique<QWebEngineView>(parent);
    webview->setPage(new QWebEnginePage(profile, webview.get()));
    return webview;
#endif
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

std::unique_ptr<QNetworkCookie> new_cookie(QByteArray const &name,
                                           QByteArray const &value) {
    return std::make_unique<QNetworkCookie>(name, value);
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

QNetworkCookieSameSite cookie_same_site(QNetworkCookie const &c) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 1, 0)
    return c.sameSitePolicy();
#else
    return Default;
#endif
}

void cookie_set_same_site(QNetworkCookie &c, QNetworkCookieSameSite s) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 1, 0)
    c.setSameSitePolicy(s);
#endif
}

std::int64_t cookie_expiration(QNetworkCookie const &c) {
    return c.expirationDate().toSecsSinceEpoch();
}

void cookie_set_expiration(QNetworkCookie &c, std::int64_t expiration) {
    c.setExpirationDate(QDateTime::fromSecsSinceEpoch(expiration));
}
