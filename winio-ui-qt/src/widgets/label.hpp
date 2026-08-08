#pragma once

#include "../common.hpp"
#include "edit.hpp"
#include <QLabel>

STATIC_CAST_ASSERT(QLabel, QWidget);

std::unique_ptr<QLabel> new_label(QWidget *parent);

void label_connect_link_activated(QLabel &w, callback_fn_t<void()> callback,
                                  std::uint8_t const *data);

void label_set_font(QLabel &w, rust::Str family, double size, bool bold,
                    bool italic);

QString label_font_family(const QLabel &w);

double label_font_size(const QLabel &w);

bool label_font_bold(const QLabel &w);

bool label_font_italic(const QLabel &w);
