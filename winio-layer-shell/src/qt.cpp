#include "qt.hpp"

#include <QByteArray>
#include <QGuiApplication>
#include <QMargins>
#include <QString>
#include <QWidget>
#include <QWindow>

#include <cstring>

#include <wayland-client.h>

#include <QtGui/qguiapplication_platform.h>
#include <QtGui/qpa/qplatformnativeinterface.h>

namespace {

constexpr const char *kInitializedProperty = "winio-layer-shell-initialized";
constexpr const char *kLayerShellInterface = "zwlr_layer_shell_v1";

// Cached real protocol-support result.
bool g_supported = false;
bool g_supported_known = false;

void registry_global(void *data, wl_registry *, uint32_t, char const *interface,
                     uint32_t) {
    if (interface && std::strcmp(interface, kLayerShellInterface) == 0) {
        *static_cast<bool *>(data) = true;
    }
}

void registry_global_remove(void *, wl_registry *, uint32_t) {}

// Probe the Wayland registry for zwlr_layer_shell_v1. This reports the real
// support state instead of assuming every Wayland compositor has it (wslg and
// GNOME, for example, do not).
bool probe_layer_shell_support() {
    if (!QGuiApplication::instance()) {
        return false;
    }
    auto *ni = QGuiApplication::platformNativeInterface();
    if (!ni) {
        return false;
    }
    auto *display = static_cast<wl_display *>(
        ni->nativeResourceForIntegration("wl_display"));
    if (!display) {
        return false;
    }

    wl_event_queue *queue = wl_display_create_queue(display);
    if (!queue) {
        return false;
    }
    wl_registry *registry = wl_display_get_registry(display);
    if (!registry) {
        wl_event_queue_destroy(queue);
        return false;
    }

    bool found = false;
    const wl_registry_listener listener = {registry_global,
                                           registry_global_remove};
    wl_registry_add_listener(registry, &listener, &found);
    wl_proxy_set_queue((struct wl_proxy *)registry, queue);
    wl_display_roundtrip_queue(display, queue);

    wl_registry_destroy(registry);
    wl_event_queue_destroy(queue);
    return found;
}

bool is_supported() {
    if (g_supported_known) {
        return g_supported;
    }
    // Only the real probe is cached; without an application we can only guess.
    if (QGuiApplication::instance()) {
        g_supported = probe_layer_shell_support();
        g_supported_known = true;
        return g_supported;
    }
    return QGuiApplication::platformName().startsWith(
        QStringLiteral("wayland"));
}

QWindow *native_window(QWidget &widget) {
    QWindow *window = widget.windowHandle();
    if (!window) {
        widget.winId();
        window = widget.windowHandle();
    }
    return window;
}

} // namespace

bool winio_layer_shell_is_supported() { return is_supported(); }

bool winio_layer_shell_is_initialized(QWidget const &widget) {
    QWindow *window = widget.windowHandle();
    return window && window->property(kInitializedProperty).toBool();
}

WinioLayerShellInit winio_layer_shell_init(QWidget &widget) {
    if (!is_supported()) {
        g_supported = false;
        g_supported_known = true;
        return WinioLayerShellInit::NotSupported;
    }
    QWindow *window = native_window(widget);
    if (!window) {
        return WinioLayerShellInit::NoWindow;
    }
    if (window->property(kInitializedProperty).toBool()) {
        g_supported = true;
        g_supported_known = true;
        return WinioLayerShellInit::Ok;
    }
    if (window->isVisible()) {
        // A visible window cannot become a layer surface: the wl_surface role
        // is fixed at creation time.
        return WinioLayerShellInit::AlreadyShown;
    }
    LayerShellQt::Window *shell = LayerShellQt::Window::get(window);
    if (!shell) {
        g_supported = false;
        g_supported_known = true;
        return WinioLayerShellInit::NoWindow;
    }
    // Apply the documented defaults (empty anchors, no keyboard focus).
    shell->setAnchors(LayerShellQt::Window::AnchorNone);
    shell->setKeyboardInteractivity(
        LayerShellQt::Window::KeyboardInteractivityNone);
    // Unify close-on-dismiss behavior with the GTK backend.
    shell->setCloseOnDismissed(true);

    // Submit an initial buffer to ensure the surface is mapped by the compositor.
    widget.update();

    // Note: We no longer set kInitializedProperty here. 
    // It should be set by the event filter when the Configure event is received.
    g_supported = true;
    g_supported_known = true;
    return WinioLayerShellInit::Ok;
}

WinioLayerShellWindow *winio_layer_shell(QWidget &widget) {
    QWindow *window = native_window(widget);
    return window ? LayerShellQt::Window::get(window) : nullptr;
}

rust::String winio_layer_shell_scope(WinioLayerShellWindow const &window) {
    const QByteArray bytes = window.scope().toUtf8();
    return rust::String(bytes.constData(), static_cast<size_t>(bytes.size()));
}

void winio_layer_shell_set_scope(WinioLayerShellWindow &window,
                                 rust::Str scope) {
    window.setScope(
        QString::fromUtf8(scope.data(), static_cast<qsizetype>(scope.size())));
}

bool winio_layer_shell_is_configured(QWidget const &widget) {
    QWindow *window = widget.windowHandle();
    return window && window->property(kInitializedProperty).toBool();
}

void winio_layer_shell_request_refresh(QWidget &widget) { widget.update(); }
