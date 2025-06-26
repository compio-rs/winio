#pragma once

#include <QString>
#include <rust/cxx.h>

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

inline std::size_t string_len(const QString &s) { return s.size(); }
