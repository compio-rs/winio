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
    WinioMediaPlayer();
    ~WinioMediaPlayer() override;

    double volume() const { return m_audio.volume(); }
    void setVolume(double v) { m_audio.setVolume(v); }

    bool isMuted() const { return m_audio.isMuted(); }
    void setMuted(bool v) { m_audio.setMuted(v); }
};
#else
using WinioMediaPlayer = QMediaPlayer;
#endif

std::unique_ptr<QVideoWidget> new_video(QWidget *parent);
std::unique_ptr<WinioMediaPlayer> new_player();

void player_set_source(WinioMediaPlayer &player, const QUrl &url);
QUrl player_get_source(WinioMediaPlayer const &player);
void player_set_output(WinioMediaPlayer &player, QVideoWidget *w);
