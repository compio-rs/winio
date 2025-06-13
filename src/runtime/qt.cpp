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
    QObject::connect(&m_notifier, &QSocketNotifier::activated, []() {});
    m_notifier.setEnabled(true);
}

void WinioQtEventLoop::process() {
    auto dispatcher = QApplication::eventDispatcher();
    dispatcher->processEvents(
#if QT_VERSION >= QT_VERSION_CHECK(6, 3, 0)
        QEventLoop::ApplicationExec |
#endif
        QEventLoop::WaitForMoreEvents | QEventLoop::EventLoopExec);
}

void WinioQtEventLoop::process(int maxtime) {
    auto dispatcher = QApplication::eventDispatcher();
    auto id = dispatcher->registerTimer(maxtime, Qt::CoarseTimer, qApp);
    dispatcher->processEvents(
#if QT_VERSION >= QT_VERSION_CHECK(6, 3, 0)
        QEventLoop::ApplicationExec |
#endif
        QEventLoop::WaitForMoreEvents | QEventLoop::EventLoopExec);
    dispatcher->unregisterTimer(id);
}
