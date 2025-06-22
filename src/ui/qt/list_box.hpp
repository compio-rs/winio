#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_list_widget(QWidget *parent);
void list_widget_connect_select(QWidget &w, callback_fn_t<void()> callback,
                                std::uint8_t const *data);

bool list_widget_is_selected(QWidget const &w, int i);
void list_widget_set_selected(QWidget &w, int i, bool v);

void list_widget_insert(QWidget &w, int i, rust::Str s);
void list_widget_remove(QWidget &w, int i);
void list_widget_clear(QWidget &w);
int list_widget_count(QWidget const &w);
rust::String list_widget_get(QWidget const &w, int i);
void list_widget_set(QWidget &w, int i, rust::Str s);
