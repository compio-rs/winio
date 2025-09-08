#pragma once

#include "common.hpp"
#include <QMediaPlayer>
#include <QUrl>
#include <QVideoWidget>

#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
#include <QAudioOutput>
#endif

namespace rust {
template <> struct IsRelocatable<QUrl> : std::true_type {};
} // namespace rust

inline QUrl new_url(const QString &s) { return QUrl{s}; }

inline QString url_to_qstring(const QUrl &url) { return url.toString(); }

#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
struct WinioMediaPlayer : public QMediaPlayer {
private:
    QAudioOutput m_audio;

public:
    WinioMediaPlayer() : QMediaPlayer(), m_audio() { setAudioOutput(&m_audio); }
    ~WinioMediaPlayer() override { setAudioOutput(nullptr); }

    double volume() const { return m_audio.volume(); }
    void setVolume(double v) { m_audio.setVolume(v); }

    bool isMuted() const { return m_audio.isMuted(); }
    void setMuted(bool v) { m_audio.setMuted(v); }

    void setVideoOutput(QVideoWidget *w) { QMediaPlayer::setVideoOutput(w); }
};
#else
struct WinioMediaPlayer : public QMediaPlayer {
    WinioMediaPlayer() : QMediaPlayer() {}
    ~WinioMediaPlayer() override {}

    double volume() const { return ((double)QMediaPlayer::volume()) / 100.0; }
    void setVolume(double v) { QMediaPlayer::setVolume(v * 100.0); }

    QUrl source() const { return media().canonicalUrl(); }
    void setSource(const QUrl &url) { setMedia(url); }
};
#endif

std::unique_ptr<QVideoWidget> new_video(QWidget *parent);
std::unique_ptr<WinioMediaPlayer> new_player();

void player_connect_notify(WinioMediaPlayer &p,
                           callback_fn_t<void(bool)> callback,
                           std::uint8_t const *data);
