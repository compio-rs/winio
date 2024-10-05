#include <QApplication>

struct WinioQtEventLoop {
    int m_argc;
    char **m_argv;
    QApplication m_app;

    WinioQtEventLoop();

    void add_fd(int fd) const;
    void process() const;
    void process(int maxtime) const;
};

std::unique_ptr<WinioQtEventLoop> new_event_loop();
