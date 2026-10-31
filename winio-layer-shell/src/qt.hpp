#pragma once

#include <cstdint>

#include <rust/cxx.h>

class QWidget;

bool winio_layer_shell_is_supported();

int32_t winio_layer_shell_init(QWidget *widget);
int32_t winio_layer_shell_layer(QWidget *widget);
void winio_layer_shell_set_layer(QWidget *widget, int32_t layer);
uint32_t winio_layer_shell_anchors(QWidget *widget);
void winio_layer_shell_set_anchors(QWidget *widget, uint32_t anchors);
int32_t winio_layer_shell_margin(QWidget *widget, int32_t edge);
void winio_layer_shell_set_margin(QWidget *widget, int32_t edge,
                                  int32_t margin);
int32_t winio_layer_shell_exclusive_zone(QWidget *widget);
void winio_layer_shell_set_exclusive_zone(QWidget *widget, int32_t zone);
int32_t winio_layer_shell_keyboard_interactivity(QWidget *widget);
void winio_layer_shell_set_keyboard_interactivity(QWidget *widget,
                                                  int32_t mode);
rust::String winio_layer_shell_namespace(QWidget *widget);
void winio_layer_shell_set_namespace(QWidget *widget, rust::Str namespace_);
bool winio_layer_shell_is_configured(QWidget *widget);
void winio_layer_shell_request_refresh(QWidget *widget);
rust::String winio_layer_shell_output_name(QWidget *widget);
