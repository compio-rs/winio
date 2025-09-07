#include "media.hpp"

#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
WinioMediaPlayer::WinioMediaPlayer() : QMediaPlayer(), m_audio() {
    setAudioOutput(&m_audio);
}

WinioMediaPlayer::~WinioMediaPlayer() { setAudioOutput(nullptr); }
#endif

std::unique_ptr<QVideoWidget> new_video(QWidget *parent) {
    auto w = std::make_unique<QVideoWidget>(parent);
    w->setAspectRatioMode(Qt::KeepAspectRatio);
    return w;
}

std::unique_ptr<WinioMediaPlayer> new_player() {
    return std::make_unique<WinioMediaPlayer>();
}

void player_set_source(WinioMediaPlayer &player, const QUrl &url) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
    player.setSource(url);
#else
    player.setMedia(url);
#endif
}

QUrl player_get_source(WinioMediaPlayer const &player) {
#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
    return player.source();
#else
    return player.media().canonicalUrl();
#endif
}

void player_set_output(WinioMediaPlayer &player, QVideoWidget *w) {
    player.setVideoOutput(w);
}
