#pragma once

#include "common.hpp"
#include <QWebEngineCookieStore>
#include <QWebEngineHistory>
#include <QWebEngineProfile>
#include <QWebEngineView>

STATIC_CAST_ASSERT(QWebEngineView, QWidget);

std::unique_ptr<QWebEngineView> new_webview(QWebEngineProfile *profile,
                                            QWidget *parent);

void webview_connect_load_started(QWebEngineView &w,
                                  callback_fn_t<void()> callback,
                                  std::uint8_t const *data);

void webview_connect_load_finished(QWebEngineView &w,
                                   callback_fn_t<void()> callback,
                                   std::uint8_t const *data);

void webview_page_run_js(QWebEnginePage &page, QString const &js,
                         callback_fn_t<void(QString const &)> callback,
                         std::uint8_t const *data);

std::unique_ptr<QWebEngineProfile> new_webview_profile();

std::unique_ptr<QNetworkCookie> new_cookie(QByteArray const &name,
                                           QByteArray const &value);

inline void webview_cookie_store_add(QWebEngineCookieStore &store,
                                     QNetworkCookie const &cookie) {
    store.setCookie(cookie);
}

inline void webview_cookie_store_delete(QWebEngineCookieStore &store,
                                        QNetworkCookie const &cookie) {
    store.deleteCookie(cookie);
}

void webview_cookie_store_connect_add(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data);

void webview_cookie_store_connect_delete(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data);

#if QT_VERSION >= QT_VERSION_CHECK(6, 1, 0)
using QNetworkCookieSameSite = QNetworkCookie::SameSite;
#else
enum QNetworkCookieSameSite {
    Default,
    None,
    Lax,
    Strict,
};
#endif

QNetworkCookieSameSite cookie_same_site(QNetworkCookie const &c);
void cookie_set_same_site(QNetworkCookie &c, QNetworkCookieSameSite s);

std::int64_t cookie_expiration(QNetworkCookie const &c);
void cookie_set_expiration(QNetworkCookie &c, std::int64_t expiration);
