#include "qt.hpp"

#include <QByteArray>
#include <QCoreApplication>
#include <QEvent>
#include <QGuiApplication>
#include <QMargins>
#include <QString>
#include <QWidget>
#include <QWindow>

namespace {

constexpr const char *kLayerWindowProperty = "winio-layer-shell-layer-window";
constexpr const char *kShownProperty = "winio-layer-shell-shown";
constexpr const char *kShownFilterProperty = "winio-layer-shell-shown-filter";

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

QWindow *native_window(QWidget &widget) {
    QWindow *window = widget.windowHandle();
    if (!window) {
        widget.winId();
        window = widget.windowHandle();
    }
    return window;
}

} // namespace

wl_display *winio_layer_shell_wayland_display() {
    auto *app = qobject_cast<QGuiApplication *>(QCoreApplication::instance());
    if (!app) {
        return nullptr;
    }
    auto *wayland_app =
        app->nativeInterface<QNativeInterface::QWaylandApplication>();
    if (!wayland_app) {
        return nullptr;
    }
    return wayland_app->display();
}

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
