#include "qt.hpp"

#include <QAbstractEventDispatcher>

std::unique_ptr<WinioQtEventLoop> new_event_loop(rust::Vec<rust::String> args,
                                                 int fd) {
    return std::make_unique<WinioQtEventLoop>(args, fd);
}

static std::vector<const char *> args_ptr(rust::Vec<rust::String> const &args) {
    auto result = std::vector<const char *>{};
    for (auto &arg : args) {
        result.push_back(arg.data());
    }
    return result;
}

WinioQtEventLoop::WinioQtEventLoop(rust::Vec<rust::String> args, int fd)
    : m_args(std::move(args)), m_args_ptr(args_ptr(m_args)),
      m_argc(m_args.size()), m_app{m_argc, (char **)m_args_ptr.data()},
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
