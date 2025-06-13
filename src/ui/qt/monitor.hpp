#pragma once

#include <QScreen>

#include <rust/cxx.h>
#include <winio/src/ui/qt/monitor.rs.h>

rust::Vec<Monitor> screen_all();
