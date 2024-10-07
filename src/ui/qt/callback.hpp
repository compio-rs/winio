#pragma once

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
