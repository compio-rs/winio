#pragma once

#include <QApplication>
#include <QSocketNotifier>
#include <rust/cxx.h>
#include <vector>

struct WinioQtEventLoop {
    rust::Vec<rust::String> m_args;
    std::vector<const char *> m_args_ptr;
    int m_argc;
    QApplication m_app;
    QSocketNotifier m_notifier;

    WinioQtEventLoop(rust::Vec<rust::String> args, int fd);

    void process();
    void process(int maxtime);
};

std::unique_ptr<WinioQtEventLoop> new_event_loop(rust::Vec<rust::String> args,
                                                 int fd);