#include "qt.hpp"

#include <QAbstractEventDispatcher>
#include <QString>

std::unique_ptr<WinioQtEventLoop> new_event_loop(rust::Vec<rust::String> args) {
    return std::make_unique<WinioQtEventLoop>(args);
}

static std::vector<const char *> args_ptr(rust::Vec<rust::String> const &args) {
    auto result = std::vector<const char *>{};
    for (auto &arg : args) {
        result.push_back(arg.data());
    }
    return result;
}

WinioQtEventLoop::WinioQtEventLoop(rust::Vec<rust::String> args)
    : m_args(std::move(args)), m_args_ptr(args_ptr(m_args)),
      m_argc(m_args.size()), m_app{m_argc, (char **)m_args_ptr.data()},
      m_notifier{std::nullopt}, m_timer{std::nullopt} {
    QApplication::setQuitOnLastWindowClosed(false);
}

void WinioQtEventLoop::registerFd(int fd, int timeout,
                                  rust::Fn<void()> callback) {
    if (timeout == 0) {
        callback();
        return;
    }

    m_notifier.emplace(fd, QSocketNotifier::Read);
    if (timeout < 0) {
        QObject::connect(&*m_notifier, &QSocketNotifier::activated,
                         [callback]() { callback(); });
        m_notifier->setEnabled(true);
    } else {
        m_timer.emplace();
        m_timer->setSingleShot(true);
        m_timer->setInterval(timeout);
        QObject::connect(&*m_notifier, &QSocketNotifier::activated,
                         [this, callback]() {
                             if (m_timer)
                                 m_timer->stop();
                             callback();
                         });
        QObject::connect(&*m_timer, &QTimer::timeout, [this, callback]() {
            if (m_notifier)
                m_notifier->setEnabled(false);
            callback();
        });
        m_notifier->setEnabled(true);
        m_timer->start();
    }
}

void WinioQtEventLoop::unregisterFd() {
    m_notifier.reset();
    m_timer.reset();
}

void WinioQtEventLoop::setAppName(rust::Str name) {
    m_app.setDesktopFileName(QString::fromUtf8(name.data(), name.size()));
}

void event_loop_wake_up() {
    auto dispatcher = QApplication::eventDispatcher();
    dispatcher->wakeUp();
}

void event_loop_process() {
    auto dispatcher = QApplication::eventDispatcher();
    dispatcher->processEvents(
#if QT_VERSION >= QT_VERSION_CHECK(6, 3, 0)
        QEventLoop::ApplicationExec |
#endif
        QEventLoop::WaitForMoreEvents | QEventLoop::EventLoopExec);
}
