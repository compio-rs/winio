#pragma once

#include "common.hpp"
#include <QAudioOutput>
#include <QMediaPlayer>
#include <QUrl>
#include <QVideoWidget>

namespace rust {
template <> struct IsRelocatable<QUrl> : std::true_type {};
} // namespace rust

inline QUrl new_url(const QString &s) { return QUrl{s}; }

inline QString url_to_qstring(const QUrl &url) { return url.toString(); }

std::unique_ptr<QVideoWidget> new_video(QWidget *parent);
std::unique_ptr<QAudioOutput> new_audio();
std::unique_ptr<QMediaPlayer> new_player(const QUrl &url);

void player_set_output(QMediaPlayer &player, QWidget *w);
