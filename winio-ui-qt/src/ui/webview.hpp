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

void webview_cookie_store_add(QWebEngineCookieStore &store, rust::Str cookies);

void webview_cookie_store_delete(QWebEngineCookieStore &store,
                                 rust::Str cookies);

void webview_cookie_store_connect_add(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data);

void webview_cookie_store_connect_delete(
    QWebEngineCookieStore &store,
    callback_fn_t<void(QNetworkCookie const &)> callback,
    std::uint8_t const *data);

QString cookie_to_raw(QNetworkCookie const &cookie);
