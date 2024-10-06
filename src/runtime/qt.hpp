#pragma once

#include <QApplication>
#include <QSocketNotifier>

struct WinioQtEventLoop {
    int m_argc;
    char **m_argv;
    QApplication m_app;
    QSocketNotifier m_notifier;

    WinioQtEventLoop(int fd);
    ~WinioQtEventLoop();

    void process() const;
    void process(int maxtime) const;
};

std::unique_ptr<WinioQtEventLoop> new_event_loop(int fd);
