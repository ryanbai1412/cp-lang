//! Optionals packed into a spare value (a niche) of what they hold, the
//! unit and either types.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "unit",
        needs: &[],
        code: "\
// `()` where C++ needs a value: in a tuple, a list or a parameter.
struct Unit {
    friend bool operator==(Unit, Unit) { return true; }
    friend bool operator!=(Unit, Unit) { return false; }
    friend bool operator<(Unit, Unit) { return false; }
    friend bool operator>(Unit, Unit) { return false; }
    friend bool operator<=(Unit, Unit) { return true; }
    friend bool operator>=(Unit, Unit) { return true; }
};
",
    },
    Piece {
        name: "niche",
        needs: &[],
        code: "\
// `?V` in one scalar S: `none` is a value no V is stored as. `none` sorts first.
template <class T, class S, S NONE> struct Niche {
    S raw = NONE;
    Niche() = default;
    explicit Niche(T x) : raw((S)x) {}
    // U: a `List<Bool>` hands back `std::optional<char>`.
    template <class U> explicit Niche(const std::optional<U>& o) : raw(o ? (S)(T)*o : NONE) {}
    bool has_value() const { return raw != NONE; }
    explicit operator bool() const { return raw != NONE; }
    T operator*() const { return (T)raw; }
    template <class U = S, class = std::enable_if_t<std::is_same<U, T>::value>> T& operator*() { return raw; }
    explicit operator std::optional<T>() const { return *this ? std::optional<T>(**this) : std::nullopt; }
    friend bool operator==(Niche a, Niche b) { return a.raw == b.raw; }
    friend bool operator!=(Niche a, Niche b) { return a.raw != b.raw; }
    friend bool operator<(Niche a, Niche b) { return b && (!a || *a < *b); }
};
// A position is an ordinal from 0: -1 is free.
using OptPos = Niche<int, int, -1>;
",
    },
    Piece {
        name: "niche_bool",
        needs: &["niche"],
        code: "\
// One byte: 0, 1, or 2 for `none`.
using OptBool = Niche<bool, u8, 2>;
",
    },
    Piece {
        name: "niche_char",
        needs: &["niche"],
        code: "\
// A Char is any byte, so `none` is the first value past one: -1 in 16 bits.
using OptChar = Niche<u8, short, -1>;
",
    },
    Piece {
        name: "niche_tuple",
        needs: &[],
        code: "\
// `?(.., V, ..)`: a tuple that holds a position, here or in a tuple inside
// it (`P` is the path to it), is `none` when that position is -1, so there
// is no flag beside the tuple.
template <size_t K, size_t... P> struct NichePath {
    template <class T> static auto& at(T& t) {
        if constexpr (sizeof...(P) == 0) return std::get<K>(t);
        else return NichePath<P...>::at(std::get<K>(t));
    }
};
template <class T, size_t... P> struct OptTuple {
    T raw = empty();
    static T empty() { T t{}; NichePath<P...>::at(t) = -1; return t; }
    OptTuple() = default;
    explicit OptTuple(const T& x) : raw(x) {}
    explicit OptTuple(const std::optional<T>& o) : raw(o ? *o : empty()) {}
    bool has_value() const { return NichePath<P...>::at(raw) != -1; }
    explicit operator bool() const { return has_value(); }
    const T& operator*() const { return raw; }
    T& operator*() { return raw; }
    explicit operator std::optional<T>() const { return has_value() ? std::optional<T>(raw) : std::nullopt; }
    friend bool operator==(const OptTuple& a, const OptTuple& b) { return a.has_value() == b.has_value() && (!a.has_value() || a.raw == b.raw); }
    friend bool operator!=(const OptTuple& a, const OptTuple& b) { return !(a == b); }
    friend bool operator<(const OptTuple& a, const OptTuple& b) { return b.has_value() && (!a.has_value() || a.raw < b.raw); }
    friend bool operator>(const OptTuple& a, const OptTuple& b) { return b < a; }
    friend bool operator<=(const OptTuple& a, const OptTuple& b) { return !(b < a); }
    friend bool operator>=(const OptTuple& a, const OptTuple& b) { return !(a < b); }
};
",
    },
    Piece {
        name: "niche_field",
        needs: &[],
        code: "\
// `?S` with no flag: S's field M (an Int or a position) never holds NONE.
template <class P> struct Member;
template <class S0, class F0> struct Member<F0 S0::*> { using S = S0; using F = F0; };
template <auto M> struct OptField {
    using S = typename Member<decltype(M)>::S;
    using F = typename Member<decltype(M)>::F;
    static constexpr F NONE = sizeof(F) == 8 ? F(-0x7fffffffffffffffLL - 1) : F(-1);
    S raw = empty();
    static S empty() { S s{}; s.*M = NONE; return s; }
    OptField() = default;
    explicit OptField(const S& x) : raw(x) {}
    explicit OptField(const std::optional<S>& o) : raw(o ? *o : empty()) {}
    bool has_value() const { return raw.*M != NONE; }
    explicit operator bool() const { return has_value(); }
    const S& operator*() const { return raw; }
    S& operator*() { return raw; }
    explicit operator std::optional<S>() const { return *this ? std::optional<S>(raw) : std::nullopt; }
    friend bool operator==(const OptField& a, const OptField& b) { return a.raw.*M == b.raw.*M && (!a || a.raw == b.raw); }
    friend bool operator!=(const OptField& a, const OptField& b) { return !(a == b); }
};
",
    },
    Piece {
        name: "niche_int",
        needs: &["niche"],
        code: "\
// An Int is never INT64_MIN, so that is `none`.
using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;
",
    },
    Piece {
        name: "niche_i32",
        needs: &["niche", "niche_int"],
        code: "\
// `?lo..hi` inside an int32_t: INT32_MIN is `none`. Narrowing is spelled, widening silent.
struct OptI32 : Niche<ll, int32_t, (-0x7fffffff - 1)> {
    using Niche::Niche;
    explicit OptI32(OptInt o) { if (o) raw = (int32_t)*o; }
    OptI32& operator=(OptInt o) { raw = o ? (int32_t)*o : (-0x7fffffff - 1); return *this; }
    operator OptInt() const { return *this ? OptInt(**this) : OptInt(); }
};
",
    },
    Piece {
        name: "either",
        needs: &[],
        code: "\
template <class O> O either(O a, O b) { return a ? a : b; }
",
    },
];
