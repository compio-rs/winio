#pragma once

#include <QScreen>

#include <rust/cxx.h>
#include <winio-ui-qt/src/platform/monitor.rs.h>

rust::Vec<Monitor> screen_all();
