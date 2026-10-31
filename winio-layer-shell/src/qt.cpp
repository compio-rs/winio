#include "qt.hpp"

#include <LayerShellQt/Window>

#include <QByteArray>
#include <QGuiApplication>
#include <QMargins>
#include <QString>
#include <QWidget>
#include <QWindow>

namespace {

constexpr const char *kInitializedProperty = "winio-layer-shell-initialized";

bool is_supported() {
    return QGuiApplication::platformName().startsWith(
        QStringLiteral("wayland"));
}

QWindow *native_window(QWidget *widget) {
    QWindow *window = widget->windowHandle();
    if (!window) {
        widget->winId();
        window = widget->windowHandle();
    }
    return window;
}

LayerShellQt::Window *layer_window(QWidget *widget) {
    if (!widget) {
        return nullptr;
    }
    QWindow *window = native_window(widget);
    if (!window) {
        return nullptr;
    }
    return LayerShellQt::Window::get(window);
}

LayerShellQt::Window::Anchors to_anchors(std::uint32_t anchors) {
    LayerShellQt::Window::Anchors result;
    if (anchors & (1u << 0)) {
        result |= LayerShellQt::Window::AnchorTop;
    }
    if (anchors & (1u << 1)) {
        result |= LayerShellQt::Window::AnchorBottom;
    }
    if (anchors & (1u << 2)) {
        result |= LayerShellQt::Window::AnchorLeft;
    }
    if (anchors & (1u << 3)) {
        result |= LayerShellQt::Window::AnchorRight;
    }
    return result;
}

} // namespace

bool winio_layer_shell_is_supported() { return is_supported(); }

// The integer return values are mapped in src/qt.rs.
int32_t winio_layer_shell_init(QWidget *widget) {
    if (!is_supported()) {
        return 1;
    }
    if (!widget) {
        return 2;
    }
    QWindow *window = native_window(widget);
    if (!window) {
        return 2;
    }
    if (window->property(kInitializedProperty).toBool()) {
        return 0;
    }
    if (window->isVisible()) {
        // A window that is already visible cannot be converted into a layer
        // surface: the wl_surface role is fixed at creation time.
        return 3;
    }
    if (!LayerShellQt::Window::get(window)) {
        return 2;
    }
    window->setProperty(kInitializedProperty, true);
    return 0;
}

int32_t winio_layer_shell_layer(QWidget *widget) {
    auto *layer = layer_window(widget);
    return layer ? static_cast<int32_t>(layer->layer()) : 2;
}

void winio_layer_shell_set_layer(QWidget *widget, int32_t value) {
    if (auto *layer = layer_window(widget)) {
        layer->setLayer(static_cast<LayerShellQt::Window::Layer>(value));
    }
}

uint32_t winio_layer_shell_anchors(QWidget *widget) {
    auto *layer = layer_window(widget);
    return layer ? static_cast<uint32_t>(layer->anchors().toInt()) : 0;
}

void winio_layer_shell_set_anchors(QWidget *widget, uint32_t anchors) {
    if (auto *layer = layer_window(widget)) {
        layer->setAnchors(to_anchors(anchors));
    }
}

int32_t winio_layer_shell_margin(QWidget *widget, int32_t edge) {
    auto *layer = layer_window(widget);
    if (!layer) {
        return 0;
    }
    const QMargins margins = layer->margins();
    switch (edge) {
    case 0:
        return margins.top();
    case 1:
        return margins.bottom();
    case 2:
        return margins.left();
    default:
        return margins.right();
    }
}

void winio_layer_shell_set_margin(QWidget *widget, int32_t edge,
                                  int32_t margin) {
    auto *layer = layer_window(widget);
    if (!layer) {
        return;
    }
    QMargins margins = layer->margins();
    switch (edge) {
    case 0:
        margins.setTop(margin);
        break;
    case 1:
        margins.setBottom(margin);
        break;
    case 2:
        margins.setLeft(margin);
        break;
    default:
        margins.setRight(margin);
        break;
    }
    layer->setMargins(margins);
}

int32_t winio_layer_shell_exclusive_zone(QWidget *widget) {
    auto *layer = layer_window(widget);
    return layer ? layer->exclusionZone() : 0;
}

void winio_layer_shell_set_exclusive_zone(QWidget *widget, int32_t zone) {
    if (auto *layer = layer_window(widget)) {
        layer->setExclusiveZone(zone);
    }
}

int32_t winio_layer_shell_keyboard_interactivity(QWidget *widget) {
    auto *layer = layer_window(widget);
    return layer ? static_cast<int32_t>(layer->keyboardInteractivity()) : 0;
}

void winio_layer_shell_set_keyboard_interactivity(QWidget *widget,
                                                  int32_t mode) {
    if (auto *layer = layer_window(widget)) {
        layer->setKeyboardInteractivity(
            static_cast<LayerShellQt::Window::KeyboardInteractivity>(mode));
    }
}

rust::String winio_layer_shell_namespace(QWidget *widget) {
    auto *layer = layer_window(widget);
    if (!layer) {
        return {};
    }
    const QByteArray bytes = layer->scope().toUtf8();
    return rust::String(bytes.constData(), static_cast<size_t>(bytes.size()));
}

void winio_layer_shell_set_namespace(QWidget *widget, rust::Str namespace_) {
    if (auto *layer = layer_window(widget)) {
        layer->setScope(QString::fromUtf8(
            namespace_.data(), static_cast<qsizetype>(namespace_.size())));
    }
}

bool winio_layer_shell_is_configured(QWidget *widget) {
    if (!widget) {
        return false;
    }
    QWindow *window = widget->windowHandle();
    return window && window->property(kInitializedProperty).toBool();
}

void winio_layer_shell_request_refresh(QWidget *widget) {
    if (widget) {
        // Queue a repaint; layer-shell-qt submits a new buffer when the
        // compositor's frame callback fires.
        widget->update();
    }
}

rust::String winio_layer_shell_output_name(QWidget *widget) {
    if (!widget) {
        return {};
    }
    QWindow *window = widget->windowHandle();
    if (!window || !window->screen()) {
        return {};
    }
    const QByteArray bytes = window->screen()->name().toUtf8();
    return rust::String(bytes.constData(), static_cast<size_t>(bytes.size()));
}
