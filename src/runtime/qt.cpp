#include "qt.hpp"

#include <QAbstractEventDispatcher>

std::unique_ptr<WinioQtEventLoop> new_event_loop(int fd) {
    return std::make_unique<WinioQtEventLoop>(fd);
}

WinioQtEventLoop::WinioQtEventLoop(int fd)
    : m_argc(0), m_argv(nullptr), m_app{m_argc, m_argv},
      m_notifier{fd, QSocketNotifier::Read} {
    QApplication::setQuitOnLastWindowClosed(false);
    QApplication::eventDispatcher()->registerSocketNotifier(&m_notifier);
}

WinioQtEventLoop::~WinioQtEventLoop() {
    QApplication::eventDispatcher()->unregisterSocketNotifier(&m_notifier);
}

void WinioQtEventLoop::process() const {
    QApplication::processEvents(QEventLoop::AllEvents);
}

void WinioQtEventLoop::process(int maxtime) const {
    QApplication::processEvents(QEventLoop::AllEvents, maxtime);
}
