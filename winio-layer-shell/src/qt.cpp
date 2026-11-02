#include "qt.hpp"

#include <QByteArray>
#include <QCoreApplication>
#include <QEvent>
#include <QGuiApplication>
#include <QMargins>
#include <QString>
#include <QWidget>
#include <QWindow>

#include <cstring>

#include <wayland-client.h>

namespace {

constexpr const char *kLayerWindowProperty = "winio-layer-shell-layer-window";
constexpr const char *kShownProperty = "winio-layer-shell-shown";
constexpr const char *kShownFilterProperty = "winio-layer-shell-shown-filter";
constexpr const char *kLayerShellInterface = "zwlr_layer_shell_v1";

// Latches "this window has been shown" because QWindow only reports current
// visibility, while the layer surface role is fixed when it is first created.
class ShownFilter : public QObject {
public:
    bool eventFilter(QObject *watched, QEvent *event) override {
        if (event->type() == QEvent::Show) {
            watched->setProperty(kShownProperty, true);
        }
        return QObject::eventFilter(watched, event);
    }
};

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
    auto *app = qApp;
    auto *wayland_app =
        app->nativeInterface<QNativeInterface::QWaylandApplication>();
    if (!wayland_app) {
        return false;
    }
    wl_display *display = wayland_app->display();
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
    if (!qApp) {
        return false;
    }
    static const bool supported = probe_layer_shell_support();
    return supported;
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

bool winio_layer_shell_is_initialized(QWidget const &widget) {
    QWindow *window = widget.windowHandle();
    return window && window->property(kLayerWindowProperty).toBool();
}

bool winio_layer_shell_is_shown(QWidget const &widget) {
    QWindow *window = widget.windowHandle();
    if (!window) {
        return false;
    }
    // A window that is currently visible has obviously been shown; latch it so
    // hiding the window later does not make it look unshown again.
    if (window->isVisible()) {
        window->setProperty(kShownProperty, true);
    }
    return window->property(kShownProperty).toBool();
}

WinioLayerShellInit winio_layer_shell_init(QWidget &widget) {
    if (!is_supported()) {
        return WinioLayerShellInit::NotSupported;
    }
    QWindow *window = native_window(widget);
    if (!window) {
        return WinioLayerShellInit::NoWindow;
    }
    if (window->property(kLayerWindowProperty).toBool()) {
        return WinioLayerShellInit::Ok;
    }
    if (window->isVisible()) {
        // A visible window cannot become a layer surface: the wl_surface role
        // is fixed at creation time. Qt has no "has ever been shown" state,
        // so visibility is the only reliable signal; configure the surface
        // before calling show().
        return WinioLayerShellInit::AlreadyShown;
    }
    LayerShellQt::Window *shell = LayerShellQt::Window::get(window);
    if (!shell) {
        return WinioLayerShellInit::NoWindow;
    }
    // Apply the documented defaults (empty anchors, no keyboard focus).
    shell->setAnchors(LayerShellQt::Window::AnchorNone);
    shell->setKeyboardInteractivity(
        LayerShellQt::Window::KeyboardInteractivityNone);
    // Unify close-on-dismiss behavior with the GTK backend.
    shell->setCloseOnDismissed(true);

    // Track whether the window has been shown at least once, since its
    // namespace cannot change after the layer surface is created.
    if (!window->property(kShownFilterProperty).toBool()) {
        auto *filter = new ShownFilter();
        filter->setParent(window);
        window->installEventFilter(filter);
        window->setProperty(kShownFilterProperty, true);
    }

    // Mark the window as a layer surface so the getters work. This is *not*
    // the compositor's Configure event.
    window->setProperty(kLayerWindowProperty, true);

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
