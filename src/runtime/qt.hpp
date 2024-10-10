#pragma once

#include <QApplication>
#include <QSocketNotifier>
#include <QTimer>
#include <memory>
#include <rust/cxx.h>
#include <vector>

bool poll_runtime() noexcept;

struct WinioQtEventLoop {
    rust::Vec<rust::String> m_args;
    std::vector<const char *> m_args_ptr;
    int m_argc;
    QApplication m_app;
    QTimer m_timer;

    WinioQtEventLoop(rust::Vec<rust::String> args);
};

std::unique_ptr<WinioQtEventLoop> new_event_loop(rust::Vec<rust::String> args);
void exec();
