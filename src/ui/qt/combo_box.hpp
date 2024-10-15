#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_combo_box(QWidget *parent, bool editable);
void combo_box_connect_changed(QWidget &w, callback_fn_t<void()> callback,
                               std::uint8_t const *data);
void combo_box_connect_select(QWidget &w, callback_fn_t<void()> callback,
                              std::uint8_t const *data);

rust::String combo_box_get_text(QWidget const &w);
void combo_box_set_text(QWidget &w, rust::Str s);

int combo_box_get_current_index(QWidget const &w);
void combo_box_set_current_index(QWidget &w, int i);

void combo_box_insert(QWidget &w, int i, rust::Str s);
void combo_box_remove(QWidget &w, int i);
void combo_box_clear(QWidget &w);
int combo_box_count(QWidget const &w);
rust::String combo_box_get(QWidget const &w, int i);
void combo_box_set(QWidget &w, int i, rust::Str s);
