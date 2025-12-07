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
                     [&p, callback, data](QMediaPlayer::MediaStatus status) {
                         switch (status) {
                         case QMediaPlayer::LoadedMedia:
                             callback(data, true);
                             break;
                         case QMediaPlayer::InvalidMedia:
                             callback(data, false);
                             break;
#if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
                         case QMediaPlayer::EndOfMedia:
                             // We only need to handle loops for Qt 5 and
                             // infinite loops.
                             if (p.loops() < 0) {
                                 p.setPosition(0);
                                 p.play();
                             }
                             break;
#endif
                         default:
                             break;
                         }
                     });
}
