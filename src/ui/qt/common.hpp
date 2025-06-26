#pragma once

#include <QString>
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

#define STATIC_CAST_IMPL(t, base)                                              \
    inline base const &static_cast_##t##_##base(t const &p) noexcept {         \
        return static_cast<base const &>(p);                                   \
    }                                                                          \
    inline base &static_cast_mut_##t##_##base(t &p) noexcept {                 \
        return static_cast<base &>(p);                                         \
    }

inline QString new_string_utf8(const std::uint8_t *p, std::size_t len) {
    return QString::fromUtf8((const char *)p, (qsizetype)len);
}

inline std::size_t string_len(const QString &s) noexcept { return s.size(); }

inline void string_drop(QString &s) noexcept { s.~QString(); }
