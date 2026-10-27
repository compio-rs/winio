#include "wgpu.hpp"

#include <QMouseEvent>
#include <QOffscreenSurface>
#include <QOpenGLContext>
#include <QOpenGLFunctions>
#include <QOpenGLShaderProgram>
#include <QPainter>
#include <QWheelEvent>

#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0) && QT_CONFIG(egl)
#include <QtGui/qopenglcontext_platform.h>
#endif

#if __has_include(<EGL/egl.h>) && __has_include(<EGL/eglext.h>)
#include <EGL/egl.h>
#include <EGL/eglext.h>
#else
#define EGL_NONE 0x3038
#define EGL_WIDTH 0x3057
#define EGL_HEIGHT 0x3056
#define EGL_LINUX_DMA_BUF_EXT 0x3270
#define EGL_LINUX_DRM_FOURCC_EXT 0x3271
#define EGL_DMA_BUF_PLANE0_FD_EXT 0x3272
#define EGL_DMA_BUF_PLANE0_OFFSET_EXT 0x3273
#define EGL_DMA_BUF_PLANE0_PITCH_EXT 0x3274
#endif

// A triangle strip covering the widget, with the matching texture coordinates.
const char *const VERTEX_SHADER = R"(#version 120
attribute vec2 pos;
attribute vec2 uv;
varying vec2 v_uv;
void main() {
    v_uv = uv;
    gl_Position = vec4(pos, 0.0, 1.0);
})";

const char *const FRAGMENT_SHADER = R"(#version 120
uniform sampler2D tex;
varying vec2 v_uv;
void main() {
    gl_FragColor = texture2D(tex, v_uv);
})";

// A triangle strip covering the widget, with the matching texture coordinates.
const float QUAD[] = {
    -1.0f, -1.0f, 0.0f, 1.0f, 1.0f, -1.0f, 1.0f, 1.0f,
    -1.0f, 1.0f,  0.0f, 0.0f, 1.0f, 1.0f,  1.0f, 0.0f,
};

struct WinioWgpuCanvas::GlState {
    void *egl_display = nullptr;
    void *egl_image = nullptr;
    void *(*create_image)(void *, void *, unsigned, void *,
                          const int *) = nullptr;
    unsigned (*destroy_image)(void *, void *) = nullptr;
    void (*image_target_texture)(unsigned, void *) = nullptr;

    bool initialized = false;
    bool dmabuf_supported = false;
    bool dmabuf_ready = false;
    unsigned texture = 0;
    unsigned buffer = 0;
    std::unique_ptr<QOpenGLShaderProgram> program;
};

WinioWgpuCanvas::WinioWgpuCanvas(QWidget *parent)
    : QOpenGLWidget(parent), m_gl(std::make_unique<GlState>()),
      m_move_callback(std::nullopt), m_press_callback(std::nullopt),
      m_release_callback(std::nullopt), m_wheel_callback(std::nullopt),
      m_key_down_callback(std::nullopt), m_key_up_callback(std::nullopt),
      m_key_char_callback(std::nullopt) {
    setMouseTracking(true);
    setFocusPolicy(Qt::StrongFocus);
    // The whole area is covered by the frame.
    setAttribute(Qt::WA_OpaquePaintEvent);
}

WinioWgpuCanvas::~WinioWgpuCanvas() {
    makeCurrent();
    auto context = QOpenGLContext::currentContext();
    if (auto *functions = context ? context->functions() : nullptr) {
        if (m_gl->egl_image) {
            m_gl->destroy_image(m_gl->egl_display, m_gl->egl_image);
            m_gl->egl_image = nullptr;
        }
        if (m_gl->texture) {
            functions->glDeleteTextures(1, &m_gl->texture);
        }
        if (m_gl->buffer) {
            functions->glDeleteBuffers(1, &m_gl->buffer);
        }
    }
    m_gl->program.reset();
    doneCurrent();
}

void WinioWgpuCanvas::initializeGL() {
    auto *context = QOpenGLContext::currentContext();
    auto *functions = context->functions();
#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0) && QT_CONFIG(egl)
    if (auto *native =
            context->nativeInterface<QNativeInterface::QEGLContext>()) {
        m_gl->egl_display = native->display();
        m_gl->create_image =
            (decltype(m_gl->create_image))context->getProcAddress(
                "eglCreateImageKHR");
        m_gl->destroy_image =
            (decltype(m_gl->destroy_image))context->getProcAddress(
                "eglDestroyImageKHR");
        m_gl->image_target_texture =
            (decltype(m_gl->image_target_texture))context->getProcAddress(
                "glEGLImageTargetTexture2DOES");
        m_gl->dmabuf_supported = m_gl->egl_display && m_gl->create_image &&
                                 m_gl->destroy_image &&
                                 m_gl->image_target_texture;
    }
#endif
    if (m_gl->dmabuf_supported) {
        functions->glGenTextures(1, &m_gl->texture);
    }

    m_gl->program = std::make_unique<QOpenGLShaderProgram>();
    m_gl->program->addShaderFromSourceCode(QOpenGLShader::Vertex,
                                           VERTEX_SHADER);
    m_gl->program->addShaderFromSourceCode(QOpenGLShader::Fragment,
                                           FRAGMENT_SHADER);
    m_gl->program->bindAttributeLocation("pos", 0);
    m_gl->program->bindAttributeLocation("uv", 1);
    if (!m_gl->program->link()) {
        m_gl->program.reset();
        m_gl->dmabuf_supported = false;
    }

    m_gl->initialized = true;

    functions->glGenBuffers(1, &m_gl->buffer);
    functions->glBindBuffer(GL_ARRAY_BUFFER, m_gl->buffer);
    functions->glBufferData(GL_ARRAY_BUFFER, sizeof(QUAD), QUAD,
                            GL_STATIC_DRAW);
    functions->glBindBuffer(GL_ARRAY_BUFFER, 0);
}

void WinioWgpuCanvas::paintEvent(QPaintEvent *event) {
    if (!m_gl->initialized) {
        // No OpenGL context (or OpenGL is unusable): paint the copy like a
        // regular widget.
        paint_image();
        return;
    }
    QOpenGLWidget::paintEvent(event);
}

void WinioWgpuCanvas::paintGL() {
    auto *functions = QOpenGLContext::currentContext()->functions();

    if (m_gl->dmabuf_ready && m_gl->texture && m_gl->program) {
        functions->glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        functions->glClear(GL_COLOR_BUFFER_BIT);
        m_gl->program->bind();
        functions->glActiveTexture(GL_TEXTURE0);
        functions->glBindTexture(GL_TEXTURE_2D, m_gl->texture);
        m_gl->program->setUniformValue("tex", 0);
        functions->glBindBuffer(GL_ARRAY_BUFFER, m_gl->buffer);
        m_gl->program->enableAttributeArray(0);
        m_gl->program->setAttributeBuffer(0, GL_FLOAT, 0, 2, 4 * sizeof(float));
        m_gl->program->enableAttributeArray(1);
        m_gl->program->setAttributeBuffer(1, GL_FLOAT, 2 * sizeof(float), 2,
                                          4 * sizeof(float));
        functions->glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
        m_gl->program->disableAttributeArray(0);
        m_gl->program->disableAttributeArray(1);
        functions->glBindBuffer(GL_ARRAY_BUFFER, 0);
        functions->glBindTexture(GL_TEXTURE_2D, 0);
        m_gl->program->release();
        return;
    }

    // Fallback: draw the pixels copied back by `wgpu`.
    paint_image();
}

bool WinioWgpuCanvas::set_dmabuf(int fd, int width, int height, int stride,
                                 std::uint32_t fourcc) {
    if (!m_gl->dmabuf_supported) {
        return false;
    }

    makeCurrent();
    if (m_gl->egl_image) {
        m_gl->destroy_image(m_gl->egl_display, m_gl->egl_image);
        m_gl->egl_image = nullptr;
    }

    const int attribs[] = {
        EGL_WIDTH,
        width,
        EGL_HEIGHT,
        height,
        EGL_LINUX_DRM_FOURCC_EXT,
        static_cast<int>(fourcc),
        EGL_DMA_BUF_PLANE0_FD_EXT,
        fd,
        EGL_DMA_BUF_PLANE0_OFFSET_EXT,
        0,
        EGL_DMA_BUF_PLANE0_PITCH_EXT,
        stride,
        EGL_NONE,
    };
    m_gl->egl_image = m_gl->create_image(
        m_gl->egl_display, nullptr, EGL_LINUX_DMA_BUF_EXT, nullptr, attribs);
    if (!m_gl->egl_image) {
        m_gl->dmabuf_supported = false;
        m_gl->dmabuf_ready = false;
        doneCurrent();
        return false;
    }

    auto *functions = QOpenGLContext::currentContext()->functions();
    functions->glBindTexture(GL_TEXTURE_2D, m_gl->texture);
    m_gl->image_target_texture(GL_TEXTURE_2D, m_gl->egl_image);
    functions->glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    functions->glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    functions->glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S,
                               GL_CLAMP_TO_EDGE);
    functions->glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T,
                               GL_CLAMP_TO_EDGE);
    functions->glBindTexture(GL_TEXTURE_2D, 0);
    doneCurrent();

    m_gl->dmabuf_ready = true;
    return true;
}

void WinioWgpuCanvas::clear_dmabuf() {
    m_gl->dmabuf_ready = false;
    if (m_gl->egl_image) {
        makeCurrent();
        m_gl->destroy_image(m_gl->egl_display, m_gl->egl_image);
        m_gl->egl_image = nullptr;
        doneCurrent();
    }
}

void WinioWgpuCanvas::set_image(std::uint8_t const *data, int width, int height,
                                int stride) {
    m_gl->dmabuf_ready = false;
    m_image = QImage(data, width, height, stride, QImage::Format_ARGB32);
}

void WinioWgpuCanvas::clear_image() { m_image = QImage(); }

void WinioWgpuCanvas::paint_image() {
    QPainter painter(this);
    if (m_image.isNull()) {
        painter.fillRect(rect(), palette().window());
        return;
    }
    painter.drawImage(rect(), m_image);
}

void WinioWgpuCanvas::mouseMoveEvent(QMouseEvent *event) {
    if (m_move_callback) {
        auto &[callback, data] = *m_move_callback;
        auto pos = event->pos();
        callback(data, pos.x(), pos.y());
    }
}

void WinioWgpuCanvas::mousePressEvent(QMouseEvent *event) {
    if (m_press_callback) {
        auto &[callback, data] = *m_press_callback;
        callback(data, event->button());
    }
}

void WinioWgpuCanvas::mouseReleaseEvent(QMouseEvent *event) {
    if (m_release_callback) {
        auto &[callback, data] = *m_release_callback;
        callback(data, event->button());
    }
}

void WinioWgpuCanvas::wheelEvent(QWheelEvent *event) {
    if (m_wheel_callback) {
        auto &[callback, data] = *m_wheel_callback;
        auto delta = event->angleDelta();
        int sign = event->inverted() ? -1 : 1;
        callback(data, -delta.x() * sign, delta.y() * sign);
    }
}

void WinioWgpuCanvas::keyPressEvent(QKeyEvent *event) {
    if (m_key_down_callback) {
        auto &[callback, data] = *m_key_down_callback;
        callback(data, event->key());
    }
    if (m_key_char_callback) {
        auto &[callback, data] = *m_key_char_callback;
        callback(data, event->text());
    }
}

void WinioWgpuCanvas::keyReleaseEvent(QKeyEvent *event) {
    if (m_key_up_callback) {
        auto &[callback, data] = *m_key_up_callback;
        callback(data, event->key());
    }
}

std::unique_ptr<QWidget> new_wgpu_canvas(QWidget *parent) {
    return std::make_unique<WinioWgpuCanvas>(parent);
}

double wgpu_canvas_pixel_ratio(QWidget const &w) {
    return w.devicePixelRatioF();
}

void wgpu_canvas_set_image(QWidget const &w, std::uint8_t const *data,
                           std::uint32_t width, std::uint32_t height,
                           std::uint32_t stride) {
    auto &c =
        const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w));
    c.set_image(data, int(width), int(height), int(stride));
    c.update();
}

void wgpu_canvas_clear_dmabuf(QWidget const &w) {
    const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w))
        .clear_dmabuf();
}

void wgpu_canvas_clear_image(QWidget const &w) {
    const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w))
        .clear_image();
}

bool wgpu_canvas_set_dmabuf(QWidget const &w, int fd, std::uint32_t width,
                            std::uint32_t height, std::uint32_t stride,
                            std::uint32_t fourcc) {
    auto &c =
        const_cast<WinioWgpuCanvas &>(static_cast<WinioWgpuCanvas const &>(w));
    if (!c.set_dmabuf(fd, int(width), int(height), int(stride), fourcc)) {
        return false;
    }
    c.update();
    return true;
}

void wgpu_canvas_register_move_event(QWidget &w,
                                     callback_fn_t<void(int, int)> callback,
                                     std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_move_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_press_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_press_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_release_event(
    QWidget &w, callback_fn_t<void(QtMouseButton)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_release_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_wheel_event(QWidget &w,
                                      callback_fn_t<void(int, int)> callback,
                                      std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_wheel_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_down_event(QWidget &w,
                                         callback_fn_t<void(int)> callback,
                                         std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_down_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_up_event(QWidget &w,
                                       callback_fn_t<void(int)> callback,
                                       std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_up_callback =
        std::make_tuple(std::move(callback), data);
}

void wgpu_canvas_register_key_char_event(
    QWidget &w, callback_fn_t<void(QString const &)> callback,
    std::uint8_t const *data) {
    static_cast<WinioWgpuCanvas &>(w).m_key_char_callback =
        std::make_tuple(std::move(callback), data);
}
