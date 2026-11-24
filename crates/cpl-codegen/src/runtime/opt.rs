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
    // A negative `none` is below every value it stands for (positions and bytes are never
    // negative, Int and I32 take the least), so the order is the raw one: no branch.
    static constexpr bool none_first = NONE < S(0);
    friend bool operator<(Niche a, Niche b) {
        if constexpr (none_first) return a.raw < b.raw;
        else return b && (!a || *a < *b);
    }
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
// `?S` with no flag: the scalar S reaches through member pointers M, P... (an
// Int, a position, a range, or the inside of a packed `?` field) never holds
// NONE, the spare value after the K that the `?`s along the path keep.
template <class P> struct Member;
template <class S0, class F0> struct Member<F0 S0::*> { using S = S0; using F = F0; };
template <auto M, auto... P, class T> auto& niche_at(T& s) {
    if constexpr (sizeof...(P) == 0) return s.*M;
    else return niche_at<P...>(s.*M);
}
template <unsigned K, auto M, auto... P> struct OptField {
    using S = typename Member<decltype(M)>::S;
    using F = std::remove_reference_t<decltype(niche_at<M, P...>(std::declval<S&>()))>;
    static constexpr F NONE = std::is_unsigned<F>::value ? F(std::numeric_limits<F>::max() - K) : F(std::numeric_limits<F>::min() + K);
    S raw = empty();
    static S empty() { S s{}; niche_at<M, P...>(s) = NONE; return s; }
    OptField() = default;
    explicit OptField(const S& x) : raw(x) {}
    explicit OptField(const std::optional<S>& o) : raw(o ? *o : empty()) {}
    bool has_value() const { return niche_at<M, P...>(raw) != NONE; }
    explicit operator bool() const { return has_value(); }
    const S& operator*() const { return raw; }
    S& operator*() { return raw; }
    explicit operator std::optional<S>() const { return *this ? std::optional<S>(raw) : std::nullopt; }
    friend bool operator==(const OptField& a, const OptField& b) { return a.has_value() == b.has_value() && (!a || a.raw == b.raw); }
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
        name: "keep_max_plus",
        needs: &["niche_int", "iarith"],
        code: "\
// `for l in lo..hi: if let w = row[l]: acc[l] max= Some(base + w)`. `none` is the least raw
// value, so one branch-free pass does it and g++ can vectorise it; every add's overflow goes
// into one flag, and a pass that overflowed runs again as written, so it traps as written.
// The flag is a sign bit: an add overflowed when (base ^ s) & (w ^ s) is negative, and it made
// `none`'s INT64_MIN when (z - 1) & ~z is, z being s ^ INT64_MIN; both stay plain ops g++ vectorises.
inline void keep_max_plus(OptInt* acc, const OptInt* row, ll base, ll lo, ll hi) {
    constexpr ll NO = std::numeric_limits<ll>::min();
    ll* to = &acc[0].raw;
    const ll* from = &row[0].raw;
    ll bad = 0;
    for (ll l = lo; l < hi; l++) {
        ll w = from[l], s = (ll)((unsigned long long)base + (unsigned long long)w);
        ll z = s ^ NO;
        ll made_none = (ll)((unsigned long long)z - 1) & ~z;
        ll wrapped = (base ^ s) & (w ^ s);
        ll v = w != NO ? s : NO;
        bad |= w != NO ? (wrapped | made_none) : 0;
        to[l] = to[l] > v ? to[l] : v;
    }
    if (bad < 0)
        for (ll l = lo; l < hi; l++)
            if (row[l]) acc[l] = std::max(acc[l], OptInt(iadd(base, *row[l])));
}
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
        name: "niche_opt_opt_i32",
        needs: &["niche", "niche_int"],
        code: "\
// `??lo..hi` inside an int32_t: INT32_MIN is the outer `none`, INT32_MIN + 1 the inner one.
struct OptOptI32 {
    static constexpr int32_t NONE = -0x7fffffff - 1, INNER = -0x7fffffff;
    int32_t raw = NONE;
    OptOptI32() = default;
    explicit OptOptI32(const std::optional<OptInt>& o) : raw(!o ? NONE : *o ? (int32_t)**o : INNER) {}
    OptOptI32& operator=(const std::optional<OptInt>& o) { return *this = OptOptI32(o); }
    bool has_value() const { return raw != NONE; }
    explicit operator bool() const { return raw != NONE; }
    OptInt operator*() const { return raw == INNER ? OptInt() : OptInt((ll)raw); }
    operator std::optional<OptInt>() const { return *this ? std::optional<OptInt>(**this) : std::nullopt; }
    friend bool operator==(OptOptI32 a, OptOptI32 b) { return a.raw == b.raw; }
    friend bool operator!=(OptOptI32 a, OptOptI32 b) { return a.raw != b.raw; }
    friend bool operator<(OptOptI32 a, OptOptI32 b) { return a.raw < b.raw; }
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
