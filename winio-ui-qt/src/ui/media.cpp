#include "media.hpp"

std::unique_ptr<QVideoWidget> new_video(QWidget *parent) {
    auto w = std::make_unique<QVideoWidget>(parent);
    w->setAspectRatioMode(Qt::KeepAspectRatio);
    return w;
}

std::unique_ptr<WinioMediaPlayer> new_player() {
    return std::make_unique<WinioMediaPlayer>();
}
