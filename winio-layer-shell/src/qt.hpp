#pragma once

#include <cstdint>

#include <rust/cxx.h>

#include <LayerShellQt/Window>
#include <QWidget>

using WinioLayerShellWindow = LayerShellQt::Window;
using WinioLayerShellLayer = LayerShellQt::Window::Layer;
using WinioLayerShellKeyboardInteractivity =
    LayerShellQt::Window::KeyboardInteractivity;
using WinioLayerShellAnchors = LayerShellQt::Window::Anchors;

enum class WinioLayerShellInit : int32_t {
    Ok,
    NotSupported,
    NoWindow,
    AlreadyShown,
};

bool winio_layer_shell_is_supported();

bool winio_layer_shell_is_initialized(QWidget const &widget);

WinioLayerShellInit winio_layer_shell_init(QWidget &widget);

// LayerShellQt::Window has no public constructor and is owned by the QWindow.
WinioLayerShellWindow *winio_layer_shell(QWidget &widget);

rust::String winio_layer_shell_scope(WinioLayerShellWindow const &window);
void winio_layer_shell_set_scope(WinioLayerShellWindow &window,
                                 rust::Str scope);

rust::String winio_layer_shell_output_name(QWidget const &widget);
bool winio_layer_shell_is_configured(QWidget const &widget);

void winio_layer_shell_request_refresh(QWidget &widget);
