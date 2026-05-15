#pragma once

#include <QApplication>
#include <QSocketNotifier>
#include <QTimer>
#include <rust/cxx.h>

#include <memory>
#include <optional>
#include <vector>

struct WinioQtEventLoop {
    rust::Vec<rust::String> m_args;
    std::vector<const char *> m_args_ptr;
    int m_argc;
    QApplication m_app;
    std::optional<QSocketNotifier> m_notifier;
    std::optional<QTimer> m_timer;

    WinioQtEventLoop(rust::Vec<rust::String> args);

    void setAppName(rust::Str name);

    void registerFd(int fd, int timeout, rust::Fn<void()> callback);
    void unregisterFd();
};

std::unique_ptr<WinioQtEventLoop> new_event_loop(rust::Vec<rust::String> args);

void event_loop_wake_up();
void event_loop_process();
