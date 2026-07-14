#include "monitor.hpp"
#include <QApplication>

rust::Vec<Monitor> screen_all() {
    rust::Vec<Monitor> res{};
    for (QScreen *s : QApplication::screens()) {
        res.push_back(Monitor{s->geometry(), s->availableGeometry(),
                              s->logicalDotsPerInchX(),
                              s->logicalDotsPerInchY()});
    }
    return res;
}
