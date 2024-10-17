#pragma once

#include "callback.hpp"
#include <QWidget>
#include <memory>

std::unique_ptr<QWidget> new_push_button(QWidget *parent);
std::unique_ptr<QWidget> new_check_box(QWidget *parent);
std::unique_ptr<QWidget> new_radio_button(QWidget *parent);

void push_button_connect_clicked(QWidget &w, callback_fn_t<void()> callback,
                                 std::uint8_t const *data);

rust::String push_button_get_text(QWidget const &w);
void push_button_set_text(QWidget &w, rust::Str s);

bool check_box_is_checked(QWidget const &w);
void check_box_set_checked(QWidget &w, bool v);

bool radio_button_is_checked(QWidget const &w);
void radio_button_set_checked(QWidget &w, bool v);
