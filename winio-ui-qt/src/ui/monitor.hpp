#pragma once

#include <QScreen>

#include <rust/cxx.h>
#include <winio-ui-qt/src/ui/monitor.rs.h>

rust::Vec<Monitor> screen_all();
