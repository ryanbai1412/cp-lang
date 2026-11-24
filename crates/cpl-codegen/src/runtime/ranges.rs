//! Ranges, domains and positions: stepping, sums and counts over a range,
//! the next and previous position.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "reach",
        needs: &["fail"],
        code: "\
template <class S> decltype(auto) reach_at(const S& s, ll i) { return s[i]; }
inline u8 reach_at(const std::string& s, ll i) { return (u8)s[i]; }
template <class S, class F> std::vector<ll> reach(const S& seq, F&& pred, bool cyclic) {
    ll m = (ll)seq.size(), j = 0;
    std::vector<ll> out(m);
    for (ll i = 0; i < m; i++) {
        if (j < i + 1) j = i + 1;
#ifdef LOCAL
        if (j > i + 1) {
            ll a = cyclic ? (i + 1) % m : i + 1;
            ll b = cyclic ? (j - 1) % m : j - 1;
            if (!pred(reach_at(seq, i), reach_at(seq, a)) || (b != a && !pred(reach_at(seq, i), reach_at(seq, b))))
                fail(\"reach: `while` predicate is not monotone (reach dropped by more than one)\");
        }
#endif
        ll lim = cyclic ? i + m : m;
        while (j < lim && pred(reach_at(seq, i), reach_at(seq, cyclic ? j % m : j))) j++;
        out[i] = j - i - 1;
    }
    return out;
}
",
    },
    Piece {
        name: "next_pos",
        needs: &["niche"],
        code: "\
inline OptPos next_pos(ll v, ll len) { return v + 1 < len ? OptPos((int)(v + 1)) : OptPos(); }
",
    },
    Piece {
        name: "prev_pos",
        needs: &["niche"],
        code: "\
inline OptPos prev_pos(ll v) { return v > 0 ? OptPos((int)(v - 1)) : OptPos(); }
",
    },
    Piece {
        name: "range",
        needs: &["fail"],
        code: "\
template <class T> struct Range {
    T lo = 0, hi = 0;
    static Range make(T lo, T hi) { return {lo, hi}; }
    using U = std::conditional_t<sizeof(T) == 16, unsigned __int128, unsigned long long>;
    U span() const { return hi <= lo ? 0 : (U)hi - (U)lo; }
    ll len() const {
        if constexpr (sizeof(T) < sizeof(ll)) return (ll)span();
        if (span() > (~0ULL >> 1)) fail(\"range length overflow\");
        return (ll)span();
    }
    bool empty() const { return hi <= lo; }
    bool has(T x) const { return x >= lo && x < hi; }
    std::optional<T> first() const { return lo < hi ? std::optional<T>(lo) : std::nullopt; }
    std::optional<T> last() const { return lo < hi ? std::optional<T>((T)(hi - 1)) : std::nullopt; }
    T operator[](ll i) const { return (T)((ll)lo + i); }
    std::optional<T> get(ll i) const { return i < 0 || (U)i >= span() ? std::optional<T>() : std::optional<T>((T)(lo + (T)i)); }
    std::vector<T> rev() const {
        std::vector<T> out;
        if (lo < hi) { out.reserve((size_t)len()); for (T x = hi; x-- > lo;) out.push_back(x); }
        return out;
    }
    bool operator==(const Range& o) const { return lo == o.lo && hi == o.hi; }
    bool operator!=(const Range& o) const { return !(*this == o); }
    bool operator<(const Range& o) const { return lo != o.lo ? lo < o.lo : hi < o.hi; }
};
",
    },
    Piece {
        name: "domain_val",
        needs: &[],
        code: "\
struct DomainVal { ll lo = 0, len = 0; };
",
    },
    Piece {
        name: "dom_guard",
        needs: &[],
        code: "\
struct DomGuard {
    ll& lo; ll& len; ll saved_lo; ll saved_len;
    DomGuard(ll& lo, ll& len) : lo(lo), len(len), saved_lo(lo), saved_len(len) {}
    ~DomGuard() { lo = saved_lo; len = saved_len; }
};
",
    },
    Piece {
        name: "sym_idx",
        needs: &[],
        code: "\
inline ll sym_idx(ll a, ll b) { if (a > b) std::swap(a, b); return b * (b + 1) / 2 + a; }
",
    },
    Piece {
        name: "count_query",
        needs: &["fail"],
        code: "\
// `ask(.., limit: K)`: the judge allows `K` queries in each test case.
inline ll asked = 0;
inline void count_query(ll limit) { if (++asked > limit) fail((\"query limit exceeded: the limit is \" + std::to_string(limit)).c_str()); }
",
    },
    Piece {
        name: "steps",
        needs: &["fail"],
        code: "\
// How many values `lo, lo + by, ...` lie below `hi`, or up to it when
// `closed`. A step forward only.
inline ll steps(ll lo, ll hi, ll by, bool closed = false) { if (by <= 0) fail(\"a range steps forward: `by` is not positive\"); if (closed ? lo > hi : lo >= hi) return 0; return (ll)(((unsigned long long)hi - (unsigned long long)lo - !closed) / (unsigned long long)by) + 1; }
",
    },
    // Exact in 128 bits; past an `Int` it traps, or wraps without UB when untrapped.
    Piece {
        name: "range_sum",
        needs: &["fail"],
        code: "\
template <class R> inline ll range_sum(R r, ll by = 1) { if (by <= 0) fail(\"a range steps forward: `by` is not positive\"); if (r.lo >= r.hi) return 0; typedef __int128 S; typedef unsigned __int128 U; U n = (U)(((unsigned long long)r.hi - (unsigned long long)r.lo - 1) / (unsigned long long)by) + 1; U tri = (n & 1) ? (n - 1) / 2 * n : n / 2 * (n - 1); S sum; bool bad = __builtin_mul_overflow((S)tri, (S)by, &sum); bad |= __builtin_add_overflow(sum, (S)n * (S)r.lo, &sum); if (bad || sum < -(((S)1 << 63) - 1) || sum > ((S)1 << 63) - 1) fail(\"integer overflow: a range `sum` does not fit in an `Int` (64 bits)\"); (void)bad; return (ll)(unsigned long long)(U)sum; }
",
    },
    Piece {
        name: "range_at",
        needs: &["fail", "range"],
        code: "\
template <class T> T range_at(const Range<T>& r, ll i) { if (i < 0 || i >= r.len()) fail(\"index out of range\"); return r[i]; }
template <class T> ll key_pos(const Range<T>& r, ll k) { ll i = k - (ll)r.lo, n = r.len(); if (i < 0 || i >= n) key_fail(k, (ll)r.lo, n); return i; }
",
    },
    Piece {
        name: "pos_ops",
        needs: &["fail", "niche"],
        code: "\
inline OptPos pos_try_ord(ll i, ll len) { return (i >= 0 && i < len) ? OptPos((int)i) : OptPos(); }
inline OptPos pos_offset(ll i, ll d, ll len) { ll t = i + d; return (t >= 0 && t < len) ? OptPos((int)t) : OptPos(); }
inline int pos_cyc(ll i, ll d, ll len) { ll r = (i + d) % len; return (int)(r < 0 ? r + len : r); }
inline int bound_step(ll b, ll len) { if (b < 0 || b > len) fail(\"window leaves domain\"); return (int)b; }
inline OptPos bound_left(ll b) { return b > 0 ? OptPos((int)(b - 1)) : OptPos(); }
inline OptPos bound_right(ll b, ll len) { return b < len ? OptPos((int)b) : OptPos(); }
",
    },
];
