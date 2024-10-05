#include "qt.hpp"

#include <QAbstractEventDispatcher>
#include <QSocketNotifier>

std::unique_ptr<WinioQtEventLoop> new_event_loop() {
    return std::make_unique<WinioQtEventLoop>();
}

WinioQtEventLoop::WinioQtEventLoop()
    : m_argc(0), m_argv(nullptr), m_app{m_argc, m_argv} {
    m_app.setQuitLockEnabled(true);
}

void WinioQtEventLoop::add_fd(int fd) const {
    auto notifier = QSocketNotifier{fd, QSocketNotifier::Read};
    m_app.eventDispatcher()->registerSocketNotifier(&notifier);
}

void WinioQtEventLoop::process() const {
    m_app.processEvents(QEventLoop::AllEvents);
}

void WinioQtEventLoop::process(int maxtime) const {
    m_app.processEvents(QEventLoop::AllEvents, maxtime);
}
