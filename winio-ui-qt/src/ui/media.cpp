#include "media.hpp"

std::unique_ptr<QVideoWidget> new_video(QWidget *parent) {
    auto w = std::make_unique<QVideoWidget>(parent);
    w->setAspectRatioMode(Qt::KeepAspectRatio);
    return w;
}

std::unique_ptr<WinioMediaPlayer> new_player() {
    return std::make_unique<WinioMediaPlayer>();
}

void player_connect_notify(WinioMediaPlayer &p,
                           callback_fn_t<void(bool)> callback,
                           std::uint8_t const *data) {
    QObject::connect(&p, &QMediaPlayer::mediaStatusChanged,
                     [callback, data](QMediaPlayer::MediaStatus status) {
                         switch (status) {
                         case QMediaPlayer::LoadedMedia:
                             callback(data, true);
                             break;
                         case QMediaPlayer::InvalidMedia:
                             callback(data, false);
                             break;
                         default:
                             break;
                         }
                     });
}
