#include "media.hpp"

std::unique_ptr<QVideoWidget> new_video(QWidget *parent) {
    auto w = std::make_unique<QVideoWidget>(parent);
    w->setAspectRatioMode(Qt::KeepAspectRatio);
    return w;
}

std::unique_ptr<QAudioOutput> new_audio() {
    return std::make_unique<QAudioOutput>();
}

std::unique_ptr<QMediaPlayer> new_player(const QUrl &url) {
    auto player = std::make_unique<QMediaPlayer>();
    player->setSource(url);
    return player;
}

void player_set_output(QMediaPlayer &player, QWidget *w) {
    player.setVideoOutput(w);
}
