#pragma once

#include <QString>
#include <QUrl>
#include <optional>
#include <rust/cxx.h>
#include <tuple>

template <typename T> struct callback_traits;

template <typename Ret, typename... Args> struct callback_traits<Ret(Args...)> {
    using fn_type = rust::Fn<Ret(std::uint8_t const *, Args...)>;
    using type = std::optional<std::tuple<fn_type, std::uint8_t const *>>;
};

template <typename T> using callback_t = typename callback_traits<T>::type;
template <typename T>
using callback_fn_t = typename callback_traits<T>::fn_type;

namespace rust {
template <> struct IsRelocatable<QString> : std::true_type {};
} // namespace rust

#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
static_assert(sizeof(QString) == 3 * sizeof(std::size_t));
static_assert(sizeof(QByteArray) == 3 * sizeof(std::size_t));
#else
static_assert(sizeof(QString) == sizeof(std::size_t));
static_assert(sizeof(QByteArray) == sizeof(std::size_t));
#endif

static_assert(sizeof(QUrl) == sizeof(std::size_t));

#define STATIC_CAST_ASSERT(t, base)                                            \
    static_assert(std::is_base_of<base, t>::value &&                           \
                  std::is_convertible<t *, base *>::value &&                   \
                  std::is_polymorphic<base>::value &&                          \
                  std::is_polymorphic<t>::value);

inline QString new_string_utf8(const std::uint8_t *p, std::size_t len) {
    return QString::fromUtf8((const char *)p, (qsizetype)len);
}

inline std::size_t string_len(const QString &s) noexcept { return s.size(); }

inline void string_drop(QString &s) noexcept { s.~QString(); }

namespace rust {
template <> struct IsRelocatable<QUrl> : std::true_type {};
} // namespace rust

inline QUrl new_url(const QString &s) { return QUrl{s}; }

inline QString url_to_qstring(const QUrl &url) { return url.toString(); }

namespace rust {
template <> struct IsRelocatable<QByteArray> : std::true_type {};
} // namespace rust

inline QByteArray new_byte_array(const std::uint8_t *p, std::size_t len) {
    return QByteArray((const char *)p, (qsizetype)len);
}

inline std::uint8_t const *byte_array_data(const QByteArray &arr) noexcept {
    return (const std::uint8_t *)arr.constData();
}

inline std::size_t byte_array_len(const QByteArray &arr) noexcept {
    return arr.size();
}
