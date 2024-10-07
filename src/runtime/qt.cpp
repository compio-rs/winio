#include "qt.hpp"

#include <QAbstractEventDispatcher>
#include <QTimer>

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
      m_timer(&m_app) {
    QApplication::setQuitOnLastWindowClosed(false);

    m_timer.setSingleShot(false);
    QObject::connect(&m_timer, &QTimer::timeout, []() {
        if (poll_runtime()) {
            QApplication::exit();
        }
    });
    m_timer.start(0);
}

void exec() { QApplication::exec(); }
