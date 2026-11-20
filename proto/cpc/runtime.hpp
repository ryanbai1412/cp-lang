#include <bits/stdc++.h>
#include <unistd.h>
using namespace std;
typedef long long ll;

[[noreturn]] static void cp_fail(const char* what, int line) {
    fprintf(stderr, "line %d: %s\n", line, what);
    exit(3);
}
static inline int chk_dom(ll x, ll len, int line) {
    if (x < 0 || x >= len) cp_fail("domain value out of range", line);
    return (int)x;
}
// `i + k`, `i - k`, `v.l(k)`, `v.r(k)`: position `ord + shift + k` (or `- k`) of a domain with `len` values.
[[noreturn]] static void cp_fail_step(ll ord, ll k, ll len, const char* what, int line) {
    fprintf(stderr, "line %d: %s (%lld values): from position %lld, k = %lld\n", line, what, len, ord, k);
    exit(3);
}
static inline int chk_step(ll ord, ll shift, ll k, bool minus, ll len, const char* what, int line) {
    ll x = ord + shift + (minus ? -k : k);
    if (x < 0 || x >= len) cp_fail_step(ord, k, len, what, line);
    return (int)x;
}
// The same when `k` is not known to be small: the sum is taken in 128 bits, so a huge `k` is reported as
// leaving the domain and never wraps back into it.
static inline int chk_step_wide(ll ord, ll shift, ll k, bool minus, ll len, const char* what, int line) {
    __int128 x = (__int128)ord + shift + (minus ? -(__int128)k : (__int128)k);
    if (x < 0 || x >= len) cp_fail_step(ord, k, len, what, line);
    return (int)x;
}
static inline int chk_nonempty(ll pos, ll len, const char* what, int line) {
    if (len <= 0) cp_fail(what, line);
    return (int)pos;
}
static inline ll chk_dom_len(__int128 len, int line) {
    if (len > INT_MAX) cp_fail("domain has more than 2^31 - 1 values", line);
    return len < 0 ? 0 : (ll)len;
}
#ifdef LOCAL
static constexpr bool CP_LOCAL = true;
#define CP_ASSERT(c, msg, line) do { if (!(c)) cp_fail(msg, line); } while (0)
#else
static constexpr bool CP_LOCAL = false;
#define CP_ASSERT(c, msg, line) do {} while (0)
#endif

// Arithmetic the range checker could not bound: a local build stops on overflow (decision: unproven overflow
// traps locally only); a judge build is the plain operator.
#ifdef LOCAL
static inline ll cp_add(ll a, ll b, int line) { ll r; if (__builtin_add_overflow(a, b, &r)) cp_fail("Int overflow", line); return r; }
static inline ll cp_sub(ll a, ll b, int line) { ll r; if (__builtin_sub_overflow(a, b, &r)) cp_fail("Int overflow", line); return r; }
static inline ll cp_mul(ll a, ll b, int line) { ll r; if (__builtin_mul_overflow(a, b, &r)) cp_fail("Int overflow", line); return r; }
static inline ll cp_shl(ll a, ll b, int line) {
    if (b < 0 || b > 62 || a < 0 || a > (LLONG_MAX >> b)) cp_fail("Int overflow", line);
    return a << b;
}
static inline ll cp_shr(ll a, ll b, int line) { if (b < 0 || b > 63) cp_fail("shift amount out of range", line); return a >> b; }
static inline ll cp_abs(ll a, int line) { if (a == LLONG_MIN) cp_fail("Int overflow", line); return a < 0 ? -a : a; }
#else
#define cp_shr(a, b, line) ((ll)(a) >> (b))
#define cp_abs(a, line) llabs(a)
#define cp_add(a, b, line) ((ll)(a) + (ll)(b))
#define cp_sub(a, b, line) ((ll)(a) - (ll)(b))
#define cp_mul(a, b, line) ((ll)(a) * (ll)(b))
#define cp_shl(a, b, line) ((ll)(a) << (b))
#endif

static inline ll cp_popcount(ll x) { return __builtin_popcountll((unsigned long long)x); }
static inline ll cp_ctz(ll x) { return x == 0 ? 64 : __builtin_ctzll((unsigned long long)x); }
static inline ll cp_clz(ll x) { return x == 0 ? 64 : __builtin_clzll((unsigned long long)x); }
static inline ll cp_bit_len(ll x) { return 64 - cp_clz(x); }

// "{x:.3f}" and "{n:04}": a number under a checked format; a value that rounds to zero is never written `-0.000`
static inline string cp_fmt_f(double x, int prec, int width, bool zero) {
    char buf[512];
    snprintf(buf, sizeof buf, zero ? "%0*.*f" : "%*.*f", width, prec, x);
    string r(buf);
    bool nonzero = false;
    for (char c : r) if (c >= '1' && c <= '9') nonzero = true;
    if (!nonzero) { size_t k = r.find('-'); if (k != string::npos) { r.erase(k, 1); if ((int)r.size() < width) r.insert(0, 1, zero ? '0' : ' '); } }
    return r;
}
static inline string cp_fmt_i(ll x, int width, bool zero) {
    char buf[64];
    snprintf(buf, sizeof buf, zero ? "%0*lld" : "%*lld", width, (long long)x);
    return string(buf);
}

template <class A, class V> static inline void cp_fill(A& a, const V& v) { for (auto&& x : a) x = v; }

// Str: a flat run of ASCII bytes at positions 0..len (cheat sheet section 4)
static inline string cp_str_slice(const string& s, ll lo, ll hi, bool incl, int line) {
    if (hi < 0 && !incl) hi = (ll)s.size(); else if (incl) hi += 1;
#ifdef LOCAL
    if (lo < 0 || hi > (ll)s.size() || lo > hi) cp_fail("Str slice out of bounds", line);
#endif
    return s.substr((size_t)lo, (size_t)(hi - lo));
}
static inline string cp_str_repeat(const string& s, ll k, int line) {
    if (k < 0) cp_fail("a Str cannot repeat a negative number of times", line);
    string r; r.reserve(s.size() * (size_t)k);
    for (ll i = 0; i < k; ++i) r += s;
    return r;
}
static inline bool cp_str_starts(const string& s, const string& x, ll at) {
    return at >= 0 && at <= (ll)s.size() && s.compare((size_t)at, x.size(), x) == 0;
}
static inline vector<string> cp_str_split(const string& s, char c) {
    vector<string> r; size_t from = 0;
    for (size_t k = 0; k <= s.size(); ++k) if (k == s.size() || s[k] == c) { r.push_back(s.substr(from, k - from)); from = k + 1; }
    return r;
}
static inline string cp_str_trim_start(const string& s, char c) {
    size_t k = 0; while (k < s.size() && s[k] == c) ++k;
    return s.substr(k);
}
static inline string cp_str_join(const string& sep, const vector<string>& xs) {
    string r;
    for (size_t k = 0; k < xs.size(); ++k) { if (k) r += sep; r += xs[k]; }
    return r;
}
static inline ll cp_parse_int(const string& s, int line) {
    size_t k = 0; bool neg = false; ll v = 0;
    if (k < s.size() && (s[k] == '-' || s[k] == '+')) neg = s[k++] == '-';
    if (k == s.size()) cp_fail("parse_int: not a number", line);
    for (; k < s.size(); ++k) {
        if (s[k] < '0' || s[k] > '9') cp_fail("parse_int: not a number", line);
        if (__builtin_mul_overflow(v, 10LL, &v) || __builtin_add_overflow(v, (ll)(s[k] - '0'), &v)) cp_fail("parse_int: Int overflow", line);
    }
    return neg ? -v : v;
}

template <class T> struct Opt {
    T v{}; bool h = false;
    Opt() {}
    Opt(T x) : v(x), h(true) {}
    bool has() const { return h; }
    const T& get() const { return v; }
    bool operator==(const Opt& o) const { return h == o.h && (!h || v == o.v); }
    bool operator<(const Opt& o) const { return h && o.h ? v < o.v : h < o.h; }   // none sorts before every value
    bool operator>(const Opt& o) const { return o < *this; }
    bool operator!=(const Opt& o) const { return !(*this == o); }
};
template <> struct Opt<int> {
    int v = -1;
    Opt() {}
    Opt(int x) : v(x) {}
    bool has() const { return v >= 0; }
    int get() const { return v; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator<(const Opt& o) const { return v < o.v; }
    bool operator>(const Opt& o) const { return v > o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
template <> struct Opt<bool> {
    signed char v = -1;
    Opt() {}
    Opt(bool x) : v(x) {}
    bool has() const { return v >= 0; }
    bool get() const { return v == 1; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator<(const Opt& o) const { return v < o.v; }
    bool operator>(const Opt& o) const { return v > o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
// ?Int stores none as LLONG_MIN, so it costs no more space than Int
template <> struct Opt<ll> {
    ll v = LLONG_MIN;
    Opt() {}
    Opt(ll x) : v(x) {
#ifdef LOCAL
        if (x == LLONG_MIN) cp_fail("?Int cannot hold the minimum Int", 0);
#endif
    }
    bool has() const { return v != LLONG_MIN; }
    ll get() const { return v; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator<(const Opt& o) const { return v < o.v; }
    bool operator>(const Opt& o) const { return v > o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
template <class T> static inline T opt_unwrap(const Opt<T>& o, int line) {
    if (!o.has()) cp_fail("unwrapped none", line);
    return o.get();
}
template <class T, class U> static inline void opt_min(Opt<T>& o, U x) { if (!o.has() || T(x) < o.get()) o = Opt<T>(T(x)); }
template <class T, class U> static inline void opt_max(Opt<T>& o, U x) { if (!o.has() || o.get() < T(x)) o = Opt<T>(T(x)); }
static inline void opt_min(Opt<ll>& o, ll x) {
#ifdef LOCAL
    if (x == LLONG_MIN) cp_fail("?Int cannot hold the minimum Int", 0);
#endif
    if (o.v == LLONG_MIN || x < o.v) o.v = x;
}
static inline void opt_max(Opt<ll>& o, ll x) {
#ifdef LOCAL
    if (x == LLONG_MIN) cp_fail("?Int cannot hold the minimum Int", 0);
#endif
    if (x > o.v) o.v = x;
}
// @part OptI32
// ?T for a range type that fits in 32 bits: none is INT_MIN, 4 bytes total
struct OptI32 {
    int v = INT_MIN;
    OptI32() {}
    OptI32(ll x) : v((int)x) {}
    OptI32(const Opt<ll>& o) : v(o.has() ? (int)o.get() : INT_MIN) {}
    operator Opt<ll>() const { return has() ? Opt<ll>(v) : Opt<ll>(); }
    bool has() const { return v != INT_MIN; }
    ll get() const { return v; }
    bool operator==(const OptI32& o) const { return v == o.v; }
    bool operator<(const OptI32& o) const { return v < o.v; }
    bool operator>(const OptI32& o) const { return v > o.v; }
    bool operator!=(const OptI32& o) const { return v != o.v; }
};
static inline ll opt_unwrap(const OptI32& o, int line) {
    if (!o.has()) cp_fail("unwrapped none", line);
    return o.get();
}
template <class U> static inline void opt_min(OptI32& o, U x) { if (!o.has() || (ll)x < o.get()) o = OptI32((ll)x); }
template <class U> static inline void opt_max(OptI32& o, U x) { if (!o.has() || o.get() < (ll)x) o = OptI32((ll)x); }
// @end

// F64, U64 and I128 (cheat sheet section 1)
static inline double cp_fchk(double x, int line) {
#ifdef LOCAL
    if (x != x || x - x != 0) cp_fail("F64 result is NaN or infinite", line);
#endif
    return x;
}
static inline ll cp_f2i(double x, int line) {
    if (!(x >= -9.2e18 && x <= 9.2e18)) cp_fail("F64 does not fit an Int", line);
    return (ll)x;
}
static inline ll cp_fsign(double x, double eps) { return x > eps ? 1 : x < -eps ? -1 : 0; }
static inline string cp_wide_str(unsigned long long x) { return to_string(x); }
static inline string cp_wide_str(__int128 x) {
    if (x == 0) return "0";
    bool neg = x < 0; unsigned __int128 u = neg ? -(unsigned __int128)x : (unsigned __int128)x;
    string r; while (u) { r += (char)('0' + (int)(u % 10)); u /= 10; }
    if (neg) r += '-';
    reverse(r.begin(), r.end());
    return r;
}
static inline ll cp_narrow64(__int128 x, int line) { if (x < -(__int128)LLONG_MAX || x > (__int128)LLONG_MAX) cp_fail("value does not fit an Int", line); return (ll)x; }
static inline ll cp_narrow64(unsigned long long x, int line) { if (x > (unsigned long long)LLONG_MAX) cp_fail("value does not fit an Int", line); return (ll)x; }
static inline __int128 cp_ipow128(__int128 a, ll e, int line) {
    if (e < 0) cp_fail("negative exponent", line);
    __int128 r = 1;
    for (; e; e >>= 1) { if (e & 1) { if (__builtin_mul_overflow(r, a, &r)) cp_fail("I128 overflow", line); } if (e > 1 && __builtin_mul_overflow(a, a, &a)) cp_fail("I128 overflow", line); }
    return r;
}
// Euclidean division on the wide integers, as on Int: the remainder is never negative
static inline __int128 cp_wdiv(__int128 a, __int128 b, int line) { if (b == 0) cp_fail("division by zero", line); __int128 q = a / b; if ((a % b) < 0) q += b > 0 ? -1 : 1; return q; }
static inline __int128 cp_wdivmod(__int128 a, __int128 b, int line) { if (b == 0) cp_fail("division by zero", line); __int128 r = a % b; if (r < 0) r += b < 0 ? -b : b; return r; }
static inline unsigned long long cp_wdiv(unsigned long long a, unsigned long long b, int line) { if (b == 0) cp_fail("division by zero", line); return a / b; }
static inline unsigned long long cp_wdivmod(unsigned long long a, unsigned long long b, int line) { if (b == 0) cp_fail("division by zero", line); return a % b; }
static inline double rd_f64(int line);

// desc(x): x with its order reversed, so a MinHeap<(Int, Desc<V>)> pops the largest V among equal keys
template <class T> struct CpDesc {
    T val;
    bool operator<(const CpDesc& o) const { return o.val < val; }
    bool operator>(const CpDesc& o) const { return val < o.val; }
    bool operator==(const CpDesc& o) const { return val == o.val; }
    bool operator!=(const CpDesc& o) const { return !(val == o.val); }
};
// MonoDeque<T>(min | max): the best value among those pushed at positions >= the last drop_before
template <class T> struct CpMonoDeque {
    bool smallest; deque<pair<ll, T>> q;
    explicit CpMonoDeque(bool smallest_ = true) : smallest(smallest_) {}
    void push(ll at, const T& v) {
        while (!q.empty() && (smallest ? !(q.back().second < v) : !(v < q.back().second))) q.pop_back();
        q.emplace_back(at, v);
    }
    void drop_before(ll min_at) { while (!q.empty() && q.front().first < min_at) q.pop_front(); }
    void clear() { q.clear(); }
    Opt<T> best() const { return q.empty() ? Opt<T>() : Opt<T>(q.front().second); }
};

struct Rng {
    ll lo, hi;
    bool operator==(const Rng& o) const { return lo == o.lo && hi == o.hi; }
    bool operator!=(const Rng& o) const { return !(*this == o); }
    bool operator<(const Rng& o) const { return lo != o.lo ? lo < o.lo : hi < o.hi; }
    bool operator>(const Rng& o) const { return o < *this; }
};
static inline Opt<int> cp_step(ll ord, ll k, bool minus, ll len) {
    ll x;
    if (minus ? __builtin_sub_overflow(ord, k, &x) : __builtin_add_overflow(ord, k, &x)) return Opt<int>();
    return (unsigned long long)x < (unsigned long long)len ? Opt<int>((int)x) : Opt<int>();
}
// r.prefix(k) / r.suffix(k): the first / last k positions of a range, or all of it when it is shorter
static inline ll cp_width(ll k, int line) { if (k < 1) cp_fail("windows(k) needs k >= 1", line); return k; }
static inline Rng cp_rprefix(Rng r, ll k) { return Rng{r.lo, max(r.lo, min(r.hi, r.lo + max(k, 0LL)))}; }
static inline Rng cp_rsuffix(Rng r, ll k) { return Rng{min(r.hi, max(r.lo, r.hi - max(k, 0LL))), max(r.lo, r.hi)}; }
// span(a, b): wa / wb is 1 for a position (one cell wide) and 0 for a boundary
static inline Rng cp_span(ll a, ll wa, ll b, ll wb) { return Rng{min(a, b), max(a + wa, b + wb)}; }

// @part CpView|cp_slice|cp_row|cp_win|cp_swap_view
// A lent window of an array (plan/19 §3.5): where the block is and which positions of it are lent.
// A slice keeps the positions of its base array (lo..hi); a row starts at 0. Whether it may be written
// through is decided by the checker, so the pointer is never const here.
template <class T> struct CpView {
    T* p = nullptr; int lo = 0, hi = 0;
    CpView() {}
    CpView(T* p, ll lo, ll hi) : p(p), lo((int)lo), hi((int)(hi < lo ? lo : hi)) {}
    T& operator[](ll i) const { return p[i]; }
    T* begin() const { return p + lo; }
    T* end() const { return p + hi; }
    size_t size() const { return (size_t)(hi - lo); }
};
template <class A> static inline auto cp_slice(const A& a, Rng r, int) {
    typedef typename A::value_type T;
    return CpView<T>(const_cast<T*>(a.data()), r.lo, r.hi);
}
template <class T> static inline CpView<T> cp_slice(const CpView<T>& v, Rng r, int line) {
    if (r.lo < r.hi && (r.lo < v.lo || r.hi > v.hi)) cp_fail("range is outside the slice it is taken from", line);
    return CpView<T>(v.p, r.lo, r.hi);
}
template <class A> static inline auto cp_row(const A& g, ll r, ll w) {
    typedef typename A::value_type T;
    return CpView<T>(const_cast<T*>(g.data()) + r * w, 0, w);
}
static inline ll cp_win(ll i, ll lo, ll hi, int line) {
    if ((unsigned long long)(i - lo) >= (unsigned long long)(hi - lo)) cp_fail("position is outside the slice", line);
    return i;
}
template <class T> static inline void cp_swap_view(CpView<T> a, CpView<T> b) { swap_ranges(a.begin(), a.end(), b.begin()); }
// @end

template <ll M> struct Mod {
    using U = conditional_t<(M <= 2147483647LL), uint32_t, uint64_t>;
    U v;
    Mod(ll x = 0) { ll r = x % M; v = (U)(r < 0 ? r + M : r); }
    static Mod raw(U x) { Mod r; r.v = x; return r; }
    Mod operator+(Mod o) const { U s = v + o.v; return raw(s >= (U)M ? s - (U)M : s); }
    Mod operator-(Mod o) const { U s = v - o.v; return raw(v < o.v ? s + (U)M : s); }
    Mod operator*(Mod o) const {
        if constexpr (M <= 2147483647LL) return raw((uint32_t)((uint64_t)v * o.v % (uint64_t)M));
        else return raw((uint64_t)((__uint128_t)v * o.v % (uint64_t)M));
    }
    Mod operator-() const { return raw(v ? (U)M - v : 0); }
    Mod pow(ll e) const { Mod r(1), b = *this; for (; e; e >>= 1, b = b * b) if (e & 1) r = r * b; return r; }
    Mod inv() const { return pow(M - 2); }
    Mod operator/(Mod o) const { return *this * o.inv(); }
    Mod& operator+=(Mod o) { U s = v + o.v; v = s >= (U)M ? s - (U)M : s; return *this; }
    Mod& operator-=(Mod o) { U s = v - o.v; v = v < o.v ? s + (U)M : s; return *this; }
    Mod& operator*=(Mod o) { return *this = *this * o; }
    bool operator==(Mod o) const { return v == o.v; }
    bool operator!=(Mod o) const { return v != o.v; }
};
// ?Mod<M> packs none as ~0 in the same 4 or 8 bytes as Mod<M>
template <ll M> struct Opt<Mod<M>> {
    using U = typename Mod<M>::U;
    static constexpr U NONE = ~(U)0;
    U v = NONE;
    Opt() {}
    Opt(Mod<M> x) : v(x.v) {}
    bool has() const { return v != NONE; }
    Mod<M> get() const { return Mod<M>::raw(v); }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator<(const Opt& o) const { return has() && o.has() ? v < o.v : has() < o.has(); }
    bool operator>(const Opt& o) const { return o < *this; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};

// @part CpComb
// Comb<P>() / Comb<P>(n): lazily auto-growing factorials modulo P; choose and arrange are 0 outside 0 <= k <= n
template <class M> struct CpComb {
    mutable vector<M> f, g;
    explicit CpComb(ll n = 0) : f(1, M(1)), g(1, M(1)) { if (n > 0) ensure(n); }
    void ensure(ll k) const {
        if (k < (ll)f.size()) return;
        ll old = (ll)f.size() - 1;
        ll nw = max(k, max(old * 2, 16LL));
        f.resize((size_t)nw + 1, M(1));
        g.resize((size_t)nw + 1, M(1));
        for (ll i = old + 1; i <= nw; ++i) f[(size_t)i] = f[(size_t)i - 1] * M(i);
        g[(size_t)nw] = M(1) / f[(size_t)nw];
        for (ll i = nw; i > old + 1; --i) g[(size_t)i - 1] = g[(size_t)i] * M(i);
        if (old >= 0 && nw > old) g[(size_t)old] = g[(size_t)old + 1] * M(old + 1);
    }
    void chk(ll k, int line) const { if (k < 0) cp_fail("Comb: negative argument", line); ensure(k); }
    M fact(ll k, int line) const { chk(k, line); return f[(size_t)k]; }
    M inv_fact(ll k, int line) const { chk(k, line); return g[(size_t)k]; }
    M choose(ll n, ll k, int line) const { if (k < 0 || k > n || n < 0) return M(0); chk(n, line); return f[(size_t)n] * g[(size_t)k] * g[(size_t)(n - k)]; }
    M arrange(ll n, ll k, int line) const { if (k < 0 || k > n || n < 0) return M(0); chk(n, line); return f[(size_t)n] * g[(size_t)(n - k)]; }
};

// @end
// @part CpFenwick|CpSparse|CpLazy
// Range structures over a domain (cheat sheet section 11). Ranges arrive as half-open Rng of ordinals.
struct CpFenwick {
    int n = 0; vector<ll> t;
    CpFenwick() {}
    explicit CpFenwick(ll n_) : n((int)n_), t((size_t)n_ + 1, 0) {}
    template <class A> CpFenwick(ll n_, const A& a) : n((int)n_), t((size_t)n_ + 1, 0) {
        for (int i = 1; i <= n; ++i) { t[i] += (ll)a[i - 1]; int j = i + (i & -i); if (j <= n) t[j] += t[i]; }
    }
    void add(int i, ll d) { for (++i; i <= n; i += i & -i) t[i] += d; }
    ll pre(ll k) const { ll s = 0; for (; k > 0; k -= k & -k) s += t[k]; return s; }
    ll sum(Rng r) const { return r.lo < r.hi ? pre(r.hi) - pre(r.lo) : 0; }
};
struct CpSparse {
    vector<vector<ll>> t; ll id = 0; std::function<ll(ll, ll)> op;
    CpSparse() {}
    template <class A, class F> CpSparse(const A& a, ll id_, F f) : id(id_), op(f) {
        t.emplace_back(a.begin(), a.end());
        for (size_t k = 1; (size_t(1) << k) <= t[0].size(); ++k) {
            t.emplace_back(t[0].size() - (size_t(1) << k) + 1);
            for (size_t i = 0; i < t[k].size(); ++i) t[k][i] = op(t[k - 1][i], t[k - 1][i + (size_t(1) << (k - 1))]);
        }
    }
    ll fold(Rng r, int line) const {
        if (r.lo >= r.hi) cp_fail("SparseTable.fold of an empty range", line);
        int k = 63 - __builtin_clzll((unsigned long long)(r.hi - r.lo));
        return op(t[k][(size_t)r.lo], t[k][(size_t)(r.hi - (1LL << k))]);
    }
};
// LazySegTree with a stock tag: range add or range assign, folded by sum, min or max
struct CpLazy {
    int n = 0; bool assign = false; int kind = 0; vector<ll> v, lz; vector<char> has;
    CpLazy() {}
    CpLazy(ll n_, int assign_, int kind_) : n((int)max<ll>(n_, 1)), assign(assign_), kind(kind_), v(4 * (size_t)n, 0), lz(4 * (size_t)n, 0), has(4 * (size_t)n, 0) { real = (int)n_; }
    template <class A> CpLazy(ll n_, int assign_, int kind_, const A& a) : CpLazy(n_, assign_, kind_) { if (real) build(1, 0, n - 1, a); }
    int real = 0;
    ll join(ll a, ll b) const { return kind == 0 ? a + b : kind == 1 ? min(a, b) : max(a, b); }
    template <class A> void build(int o, int l, int r, const A& a) {
        if (l == r) { v[o] = (ll)a[l]; return; }
        int m = (l + r) / 2; build(2 * o, l, m, a); build(2 * o + 1, m + 1, r, a); v[o] = join(v[2 * o], v[2 * o + 1]);
    }
    void put(int o, int l, int r, ll x) {
        if (assign) { v[o] = kind == 0 ? x * (r - l + 1) : x; lz[o] = x; }
        else { v[o] += kind == 0 ? x * (r - l + 1) : x; lz[o] = has[o] ? lz[o] + x : x; }
        has[o] = 1;
    }
    void push(int o, int l, int r) {
        if (!has[o]) return;
        int m = (l + r) / 2; put(2 * o, l, m, lz[o]); put(2 * o + 1, m + 1, r, lz[o]); has[o] = 0; lz[o] = 0;
    }
    void apply(int o, int l, int r, int ql, int qr, ll x) {
        if (ql <= l && r <= qr) { put(o, l, r, x); return; }
        push(o, l, r); int m = (l + r) / 2;
        if (ql <= m) apply(2 * o, l, m, ql, qr, x);
        if (qr > m) apply(2 * o + 1, m + 1, r, ql, qr, x);
        v[o] = join(v[2 * o], v[2 * o + 1]);
    }
    ll fold(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) return v[o];
        push(o, l, r); int m = (l + r) / 2;
        if (qr <= m) return fold(2 * o, l, m, ql, qr);
        if (ql > m) return fold(2 * o + 1, m + 1, r, ql, qr);
        return join(fold(2 * o, l, m, ql, qr), fold(2 * o + 1, m + 1, r, ql, qr));
    }
    void apply(Rng r, ll x) { if (r.lo < r.hi) apply(1, 0, n - 1, (int)r.lo, (int)r.hi - 1, x); }
    ll fold(Rng r) { return r.lo < r.hi ? fold(1, 0, n - 1, (int)r.lo, (int)r.hi - 1) : kind == 0 ? 0 : kind == 1 ? LLONG_MAX : LLONG_MIN; }
};
// @end

// ---- output (declared before input so cp_getc can flush interactive prompts before blocking on read(2))
static string cp_obuf;
static inline void out_flush() { if (!cp_obuf.empty()) { fwrite(cp_obuf.data(), 1, cp_obuf.size(), stdout); cp_obuf.clear(); } }

// ---- input
static char cp_ibuf[1 << 16]; static size_t cp_ipos = 0, cp_ilen = 0;
static inline int cp_getc() {
    if (cp_ipos == cp_ilen) {
        if (!cp_obuf.empty()) { out_flush(); fflush(stdout); }
        cp_ilen = (size_t)max<long>(0, read(0, cp_ibuf, sizeof cp_ibuf));  /* read(2), not fread: returns what is there, so interactive reads cannot deadlock on a pipe */
        cp_ipos = 0; if (!cp_ilen) return -1;
    }
    return cp_ibuf[cp_ipos++];
}
static inline bool cp_has_more() {
    while (true) {
        if (cp_ipos == cp_ilen) {
            if (!cp_obuf.empty()) { out_flush(); fflush(stdout); }
            cp_ilen = (size_t)max<long>(0, read(0, cp_ibuf, sizeof cp_ibuf));
            cp_ipos = 0; if (!cp_ilen) return false;
        }
        if ((unsigned char)cp_ibuf[cp_ipos] > ' ') return true;
        ++cp_ipos;
    }
}
// read_line(): the next line of input. After a token read that stopped inside a line, the blank rest of
// that line is skipped first, so `let n: Int = read()` then `read_line()` gives the line after n.
static inline string rd_line() {
    bool mid = cp_ipos > 0 && cp_ibuf[cp_ipos - 1] != '\n';
    for (;;) {
        string s; int c = cp_getc(); bool blank = true;
        while (c != '\n' && c != -1) { if (c != '\r') { s += (char)c; if (c != ' ' && c != '\t') blank = false; } c = cp_getc(); }
        if (mid && blank && c != -1) { mid = false; continue; }
        return s;
    }
}
static inline ll rd_int() {
    int c = cp_getc(); while (c == ' ' || c == '\n' || c == '\r' || c == '\t') c = cp_getc();
    if (c < 0) cp_fail("unexpected end of input", 0);
    bool neg = c == '-'; if (neg) c = cp_getc();
    CP_ASSERT(c >= '0' && c <= '9', "expected a number in the input", 0);   // never a silent 0 for a word
    unsigned long long x = 0; while (c >= '0' && c <= '9') { x = x * 10 + (c - '0'); c = cp_getc(); }   // unsigned: the minimum Int has no positive twin
    return (ll)(neg ? 0 - x : x);
}
static inline char rd_char() {
    int c = cp_getc(); while (c == ' ' || c == '\n' || c == '\r' || c == '\t') c = cp_getc();
    return (char)c;
}
static inline string rd_word() {
    int c = cp_getc(); while (c == ' ' || c == '\n' || c == '\r' || c == '\t') c = cp_getc();
    string s; while (c > ' ') { s += (char)c; c = cp_getc(); } return s;
}
static inline double rd_f64(int line) {
    string w = rd_word(); char* end = nullptr;
    double x = strtod(w.c_str(), &end);
    if (w.empty() || *end) cp_fail("input is not a number", line);
    return x;
}
static inline ll rd_range(ll lo, ll hi, int line) {
    ll x = rd_int();
    CP_ASSERT(lo <= x && x <= hi, "input out of declared range", line);
    return x;
}
static inline int rd_dom(ll lo, ll len, int line) { return chk_dom(rd_int() - lo, len, line); }

// ---- generator mode (cp stress)
static mt19937_64 cp_rng(1);
static ll cp_size = 10;
static inline ll gen_int(ll lo, ll hi) {
    if (lo < 0 && hi > 0 && cp_size >= 0) {  // a signed range: stay near 0 so both signs show up
        ll a = max(lo, -cp_size), b = min(hi, cp_size);
        return a + (ll)(cp_rng() % (unsigned long long)(b - a + 1));
    }
    ll h = hi; if (cp_size >= 0 && h - lo > cp_size) h = lo + (ll)(cp_rng() % (cp_size + 1));
    if (h < lo) h = lo;
    return lo + (ll)(cp_rng() % (unsigned long long)(h - lo + 1));
}

// ---- output helpers
static inline void out_int(ll x) { char b[24]; int n = 0; bool neg = x < 0; unsigned long long u = neg ? -(unsigned long long)x : x; do b[n++] = '0' + u % 10; while (u /= 10); if (neg) cp_obuf += '-'; while (n) cp_obuf += b[--n]; }
static inline void out_str(const string& s) { cp_obuf += s; }
static inline void out_char(char c) { cp_obuf += c; }
// @part cp_ediv|cp_emod|cp_ediv_pos|cp_emod_pos
// Euclidean: a == (a / b) * b + a % b and 0 <= a % b < |b|, for every a and b the native operators would
// get wrong or leave undefined: b == 0 stops the program, and b == -1 never reaches the native `/` or `%`.
static inline ll cp_emod(ll a, ll b, int line) {
    if (b == 0) cp_fail("division by zero", line);
    if (b == -1) return 0;
    ll r = a % b;
    return r < 0 ? (b < 0 ? r - b : r + b) : r;
}
static inline ll cp_ediv(ll a, ll b, int line) {
    if (b == 0) cp_fail("division by zero", line);
    if (b == -1) { if (a == LLONG_MIN) cp_fail("Int overflow", line); return -a; }
    return a / b - (a % b < 0 ? (b < 0 ? -1 : 1) : 0);
}
static inline ll cp_emod_pos(ll a, ll b) { ll r = a % b; return r < 0 ? r + b : r; }
static inline ll cp_ediv_pos(ll a, ll b) { return a / b - (a % b < 0); }
// @end
// @part cp_cyc|cp_cyc_nonneg
static inline int cp_cyc_nonneg(int v, ll k, int n) {
    ll s;
    if (!__builtin_add_overflow((ll)v, k, &s)) {
        if (s < n) return (int)s;
        if (s < 2LL * n) return (int)(s - n);
        return (int)(s % n);
    }
    return (int)(((ll)v + k % n) % n);
}
static inline int cp_cyc(int v, ll k, int n) {
    ll s;
    if (!__builtin_add_overflow((ll)v, k, &s)) {
        if ((unsigned long long)s < (unsigned long long)n) return (int)s;
        ll r = s % n;
        return (int)(r < 0 ? r + n : r);
    }
    ll r = ((ll)v + k % n) % n;
    return (int)(r < 0 ? r + n : r);
}
// @end
static inline ll cp_ipow(ll b, ll e) { ll r = 1; while (e--) r *= b; return r; }
static inline size_t cp_lidx(ll i, size_t n, int line) { if (i < 0 || (size_t)i >= n) cp_fail("list index out of range", line); return (size_t)i; }
// Sequence-safe element indexing: `i` finishes evaluating (even if it grows `c`) before `c.size()` is checked.
template <class C> static inline auto& cp_at(C& c, ll i, int line) { if (i < 0 || (size_t)i >= c.size()) cp_fail("list index out of range", line); return c[(size_t)i]; }
template <class C> static inline const auto& cp_at(const C& c, ll i, int line) { if (i < 0 || (size_t)i >= c.size()) cp_fail("list index out of range", line); return c[(size_t)i]; }
static inline ll cp_rfirst(Rng r, int line) { if (r.lo >= r.hi) cp_fail("empty range", line); return r.lo; }
static inline ll cp_rlast(Rng r, int line) { if (r.lo >= r.hi) cp_fail("empty range", line); return r.hi - 1; }
static inline Opt<ll> cp_rfirst_opt(Rng r) { return r.lo < r.hi ? Opt<ll>(r.lo) : Opt<ll>(); }
static inline Opt<ll> cp_rlast_opt(Rng r) { return r.lo < r.hi ? Opt<ll>(r.hi - 1) : Opt<ll>(); }
static inline Opt<ll> cp_rmid_opt(Rng r) { return r.lo < r.hi ? Opt<ll>(r.lo + (r.hi - 1 - r.lo) / 2) : Opt<ll>(); }
static inline Opt<int> cp_rfirst_dom(Rng r) { return r.lo < r.hi ? Opt<int>((int)r.lo) : Opt<int>(); }
static inline Opt<int> cp_rlast_dom(Rng r) { return r.lo < r.hi ? Opt<int>((int)(r.hi - 1)) : Opt<int>(); }
static inline Opt<int> cp_rmid_dom(Rng r) { return r.lo < r.hi ? Opt<int>((int)(r.lo + (r.hi - 1 - r.lo) / 2)) : Opt<int>(); }
static inline Rng cp_rinter(Rng a, Rng b) { ll lo = max(a.lo, b.lo), hi = min(a.hi, b.hi); return {lo, max(lo, hi)}; }
struct CpMarks {
    vector<int> stamp; int gen = 1; ll cnt = 0;
    CpMarks() {}
    explicit CpMarks(ll n) : stamp((size_t)n, 0) {}
    bool add(int v) { if (stamp[v] == gen) return false; stamp[v] = gen; ++cnt; return true; }
    bool remove(int v) { if (stamp[v] != gen) return false; stamp[v] = 0; --cnt; return true; }
    bool has(int v) const { return stamp[v] == gen; }
    void clear() { if (++gen == INT_MAX) { fill(stamp.begin(), stamp.end(), 0); gen = 1; } cnt = 0; }
    ll size() const { return cnt; }
    bool empty() const { return cnt == 0; }
};
template <class T> static inline T cp_take(T& x) { T r = std::move(x); x.clear(); return r; }
template <class V> static inline void cp_absorb_vec(V& a, V& b) {
    if (a.size() < b.size()) a.swap(b);
    a.insert(a.end(), make_move_iterator(b.begin()), make_move_iterator(b.end()));
    b.clear();
}
template <class S> static inline void cp_absorb_set(S& a, S& b) {
    if (a.size() < b.size()) a.swap(b);
    for (auto&& x : b) a.insert(x);
    b.clear();
}
template <class M> static inline void cp_absorb_map(M& a, M& b) {
    if (a.size() < b.size()) a.swap(b);
    for (auto&& kv : b) a.emplace(kv.first, kv.second);
    b.clear();
}
template <class T> static inline vector<T> cp_concat(vector<T> a, const vector<T>& b) { a.insert(a.end(), b.begin(), b.end()); return a; }
template <class T> static inline T cp_pop(vector<T>& v, int line) { if (v.empty()) cp_fail("pop from empty list", line); T x = v.back(); v.pop_back(); return x; }

const ll CP_INF = (ll)4e18;
template <class G> vector<Opt<ll>> cp_dijkstra(const G& g, int src) {
    int n = (int)g.size();
    vector<ll> d(n, CP_INF);
    priority_queue<pair<ll, int>, vector<pair<ll, int>>, greater<pair<ll, int>>> pq;
    d[src] = 0; pq.push({0, src});
    while (!pq.empty()) {
        auto [du, u] = pq.top(); pq.pop();
        if (du != d[u]) continue;
        for (auto& e : g[u]) {
            auto [v, w] = e.tie();
            if (du + w < d[v]) { d[v] = du + w; pq.push({d[v], (int)v}); }
        }
    }
    vector<Opt<ll>> r(n);
    for (int i = 0; i < n; ++i) if (d[i] < CP_INF) r[i] = Opt<ll>(d[i]);
    return r;
}

template <class M, class K> auto& cp_mget(M& m, const K& k, int ln) {
    auto it = m.find(k);
    if (it == m.end()) cp_fail("key not in map", ln);
    return it->second;
}

// `m[k] f= x`: the entry for k, created as f's identity when the key is new
template <class M, class K, class V> auto& cp_mslot(M& m, const K& k, const V& init) {
    auto it = m.find(k);
    if (it == m.end()) { auto& s = m[k]; s = init; return s; }
    return it->second;
}
// `m[k] min= x` / `m[k] max= x`: a new key takes x, an old one keeps the better of the two
template <class M, class K, class V, class F> void cp_mkeep(M& m, const K& k, const V& x, F f) {
    auto it = m.find(k);
    if (it == m.end()) m[k] = x; else it->second = f(it->second, x);
}

// @part SegTree|AssignSum|AddMax|StrHash|cp_scc|cp_components|cp_condense|CpDsu
// SegTree<T>: point update, range fold with a user monoid (id, op). Prototype stand-in for std.segtree.
template <class T> struct SegTree {
    int n = 0; T id; std::function<T(const T&, const T&)> op; vector<T> t;
    SegTree() {}
    template <class F> SegTree(int n_, T id_, F f) : n(n_), id(id_), op(f), t(2 * max(n_, 1), id_) {}
    // SegTree.of(op, f): leaf i is f(i), built in O(n); refresh(i) recomputes one leaf from the same f
    std::function<T(int)> src;
    template <class F, class G> SegTree(int n_, T id_, F f, G g) : n(n_), id(id_), op(f), t(2 * max(n_, 1), id_), src(g) {
        for (int i = 0; i < n; ++i) t[i + n] = src(i);
        for (int i = n - 1; i >= 1; --i) t[i] = op(t[2 * i], t[2 * i + 1]);
    }
    void refresh(int i) { set(i, src(i)); }
    T all() const { return n ? query(0, n - 1) : id; }
    void set(int i, const T& v) { for (t[i += n] = v; i > 1; i >>= 1) t[i >> 1] = op(t[i & ~1], t[i | 1]); }
    T get(int i) const { return t[i + n]; }
    T query(int l, int r) const {  // inclusive
        T a = id, b = id;
        for (l += n, r += n + 1; l < r; l >>= 1, r >>= 1) {
            if (l & 1) a = op(a, t[l++]);
            if (r & 1) b = op(t[--r], b);
        }
        return op(a, b);
    }
    template <class P> Opt<int> max_right(int l, P pred) const {
        if (l < 0 || l >= n) return Opt<int>();
        T acc = id;
        int r = l;
        while (r < n) {
            T nxt = op(acc, t[r + n]);
            if (!pred(nxt)) break;
            acc = nxt;
            ++r;
        }
        return r == l ? Opt<int>() : Opt<int>(r - 1);
    }
    template <class P> Opt<int> min_left(int r, P pred) const {
        if (r < 0 || r >= n) return Opt<int>();
        T acc = id;
        int l = r;
        while (l >= 0) {
            T nxt = op(t[l + n], acc);
            if (!pred(nxt)) break;
            acc = nxt;
            --l;
        }
        return l == r ? Opt<int>() : Opt<int>(l + 1);
    }
};

// AddMax: add to an inclusive range, max of an inclusive range. Prototype stand-in for a lazy std.segtree.
// Lazy segment tree: assign to a range, sum of a range (stdlib/lazy.cp LazySegTree with the AssignSum action)
// Euler-tour layout uses 2*n nodes instead of 4*n (halving memory).
struct AssignSum {
    int n = 0; vector<ll> v, lz;
    AssignSum() {}
    AssignSum(int n_) : n(max(n_, 1)), v(2 * max(n_, 1)), lz(2 * max(n_, 1), LLONG_MIN) {}
    void put(int o, int l, int r, ll x) { v[o] = x * (r - l + 1); lz[o] = x; }
    void push(int o, int l, int m, int r) {
        if (lz[o] != LLONG_MIN) {
            int rc = o + 2 * (m - l + 1);
            put(o + 1, l, m, lz[o]); put(rc, m + 1, r, lz[o]); lz[o] = LLONG_MIN;
        }
    }
    void assign(int o, int l, int r, int ql, int qr, ll x) {
        if (ql <= l && r <= qr) { put(o, l, r, x); return; }
        int m = (l + r) >> 1, rc = o + 2 * (m - l + 1); push(o, l, m, r);
        if (ql <= m) assign(o + 1, l, m, ql, qr, x);
        if (qr > m) assign(rc, m + 1, r, ql, qr, x);
        v[o] = v[o + 1] + v[rc];
    }
    ll sum(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) return v[o];
        int m = (l + r) >> 1, rc = o + 2 * (m - l + 1); push(o, l, m, r);
        return (ql <= m ? sum(o + 1, l, m, ql, qr) : 0) + (qr > m ? sum(rc, m + 1, r, ql, qr) : 0);
    }
    void assign(int l, int r, ll x) { if (l <= r) assign(0, 0, n - 1, l, r, x); }
    ll sum(int l, int r) { return l <= r ? sum(0, 0, n - 1, l, r) : 0; }
};

struct AddMax {
    int n = 0; vector<ll> v, lz;
    AddMax() {}
    AddMax(int n_) : n(max(n_, 1)), v(2 * max(n_, 1)), lz(2 * max(n_, 1)) {}
    void add(int o, int l, int r, int ql, int qr, ll x) {
        if (ql <= l && r <= qr) { v[o] += x; lz[o] += x; return; }
        int m = (l + r) >> 1, rc = o + 2 * (m - l + 1);
        if (ql <= m) add(o + 1, l, m, ql, qr, x);
        if (m < qr) add(rc, m + 1, r, ql, qr, x);
        v[o] = max(v[o + 1], v[rc]) + lz[o];
    }
    ll query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) return v[o];
        int m = (l + r) >> 1, rc = o + 2 * (m - l + 1); ll best = LLONG_MIN;
        if (ql <= m) best = max(best, query(o + 1, l, m, ql, qr));
        if (m < qr) best = max(best, query(rc, m + 1, r, ql, qr));
        return best + lz[o];
    }
    void add(int l, int r, ll x) { if (l <= r) add(0, 0, n - 1, l, r, x); }
    ll query(int l, int r) const { return query(0, 0, n - 1, l, r); }
    ll top() const { return v[0]; }
};

// StrHash: polynomial hash of every substring, mod 2^61 - 1. Prototype stand-in for std.hash.
struct StrHash {
    static constexpr unsigned long long M = (1ULL << 61) - 1;
    static unsigned long long mul(unsigned long long a, unsigned long long b) {
        __uint128_t t = (__uint128_t)a * b; unsigned long long r = (unsigned long long)(t & M) + (unsigned long long)(t >> 61);
        return r >= M ? r - M : r;
    }
    vector<unsigned long long> pre, pw;
    StrHash() {}
    template <class S> explicit StrHash(const S& s) : pre(s.size() + 1), pw(s.size() + 1, 1) {
        const unsigned long long B = 0x1f3d5b79a2c4e68ULL % M;
        for (size_t i = 0; i < s.size(); ++i) {
            pw[i + 1] = mul(pw[i], B);
            pre[i + 1] = mul(pre[i], B) + (unsigned long long)(unsigned char)s[i] + 1; if (pre[i + 1] >= M) pre[i + 1] -= M;
        }
    }
    ll of(int l, int r) const {  // inclusive
        unsigned long long x = pre[r + 1] + M - mul(pre[l], pw[r + 1 - l]);
        return (ll)(x >= M ? x - M : x);
    }
};

// Strongly connected components (Tarjan, iterative). Returns their number; comp[v] is numbered so that
// every arc goes to an equal or larger component (a topological order of the condensation).
template <class G, class F> static ll cp_scc(const G& g, vector<int>& comp, F to) {
    int n = (int)g.size(), timer = 0, k = 0;
    vector<int> seen(n, -1), low(n), stk;
    vector<pair<int, int>> path;
    comp.assign(n, -1);
    for (int r = 0; r < n; ++r) {
        if (seen[r] != -1) continue;
        path.push_back({r, 0});
        while (!path.empty()) {
            int u = path.back().first, i = path.back().second;
            if (i == 0) { seen[u] = low[u] = timer++; stk.push_back(u); }
            if (i < (int)g[u].size()) {
                path.back().second++;
                int v = to(g[u][i]);
                if (seen[v] == -1) path.push_back({v, 0});
                else if (comp[v] == -1) low[u] = min(low[u], seen[v]);
            } else {
                path.pop_back();
                if (!path.empty()) low[path.back().first] = min(low[path.back().first], low[u]);
                if (low[u] == seen[u]) {
                    for (;;) { int v = stk.back(); stk.pop_back(); comp[v] = k; if (v == u) break; }
                    ++k;
                }
            }
        }
    }
    for (int& c : comp) c = k - 1 - c;
    return k;
}
// Connected components of an undirected adjacency, numbered by smallest member.
template <class G, class F> static ll cp_components(const G& g, vector<int>& comp, F to) {
    int n = (int)g.size(), k = 0;
    vector<int> stk;
    comp.assign(n, -1);
    for (int r = 0; r < n; ++r) {
        if (comp[r] != -1) continue;
        comp[r] = k; stk.push_back(r);
        while (!stk.empty()) {
            int u = stk.back(); stk.pop_back();
            for (const auto& a : g[u]) { int v = to(a); if (comp[v] == -1) { comp[v] = k; stk.push_back(v); } }
        }
        ++k;
    }
    return k;
}
template <class G, class F> static vector<vector<int>> cp_condense(const G& g, const vector<int>& comp, ll k, F to) {
    vector<vector<int>> out(k);
    for (int u = 0; u < (int)g.size(); ++u)
        for (const auto& a : g[u]) if (comp[u] != comp[to(a)]) out[comp[u]].push_back(comp[to(a)]);
    for (auto& o : out) { sort(o.begin(), o.end()); o.erase(unique(o.begin(), o.end()), o.end()); }
    return out;
}

// Disjoint sets: union by size, path halving
struct CpDsu {
    vector<int> p, sz; ll parts = 0;
    vector<pair<int, int>> hist;
    bool rb = false;
    CpDsu() {}
    explicit CpDsu(ll n) : p(n), sz(n, 1), parts(n) { iota(p.begin(), p.end(), 0); }
    int find(int v) {
        if (rb) { while (p[v] != v) v = p[v]; return v; }
        while (p[v] != v) v = p[v] = p[p[v]]; return v;
    }
    bool unite(int a, int b) {
        a = find(a); b = find(b);
        if (a == b) return false;
        if (sz[a] < sz[b]) swap(a, b);
        if (rb) hist.push_back({b, sz[a]});
        p[b] = a; sz[a] += sz[b]; --parts; return true;
    }
    int mark() { rb = true; return (int)hist.size(); }
    void undo_to(int snap) {
        while ((int)hist.size() > snap) {
            auto [b, old_sz] = hist.back(); hist.pop_back();
            int a = p[b]; p[b] = b; sz[a] = old_sz; ++parts;
        }
    }
    bool same(int a, int b) { return find(a) == find(b); }
    int size(int v) { return sz[find(v)]; }
};
// @end
template <class F> struct CpDefer { F f; ~CpDefer() { f(); } };
template <class F> static inline CpDefer<F> cp_defer(F f) { return CpDefer<F>{f}; }
static inline unsigned long long cp_parse_u64(const string& s, int line) {
    char* end = nullptr; errno = 0;
    unsigned long long v = strtoull(s.c_str(), &end, 10);
    if (s.empty() || *end || errno == ERANGE) cp_fail("parse::<U64>: invalid unsigned integer", line);
    return v;
}
static inline double cp_parse_f64(const string& s, int line) {
    char* end = nullptr;
    double v = strtod(s.c_str(), &end);
    if (s.empty() || *end) cp_fail("parse::<F64>: invalid float", line);
    return v;
}

// RootedTree: the depth-first spanning tree of `adj` from `root`, with parent, preorder, O(1) lca and level
// ancestors. On a tree that is the tree itself; on any other connected graph every edge left out joins a
// vertex to one of its ancestors. Prototype stand-in for std.tree.
// @part RootedTree
struct KidSpan {
    const int *b, *e;
    const int* begin() const { return b; }
    const int* end() const { return e; }
    reverse_iterator<const int*> rbegin() const { return reverse_iterator<const int*>(e); }
    reverse_iterator<const int*> rend() const { return reverse_iterator<const int*>(b); }
    size_t size() const { return (size_t)(e - b); }
    bool empty() const { return b == e; }
    int operator[](size_t i) const { return b[i]; }
    int back() const { return *(e - 1); }
    operator vector<int>() const { return vector<int>(b, e); }
};
struct KidsCSR {
    vector<int> head, to;
    KidSpan operator[](int u) const { return {to.data() + head[u], to.data() + head[u + 1]}; }
    size_t size() const { return head.empty() ? 0 : head.size() - 1; }
    operator vector<vector<int>>() const {
        size_t n = size(); vector<vector<int>> out(n);
        for (size_t u = 0; u < n; ++u) out[u].assign(to.data() + head[u], to.data() + head[u + 1]);
        return out;
    }
};
template <class P> struct ChainList {
    P a[68]; int n = 0;
    void push_back(const P& x) { a[n++] = x; }
    const P* begin() const { return a; }
    const P* end() const { return a + n; }
    reverse_iterator<const P*> rbegin() const { return reverse_iterator<const P*>(a + n); }
    reverse_iterator<const P*> rend() const { return reverse_iterator<const P*>(a); }
    size_t size() const { return (size_t)n; }
    bool empty() const { return n == 0; }
    const P& operator[](size_t i) const { return a[i]; }
    operator vector<P>() const { return vector<P>(a, a + n); }
};
struct RootedTree {
    vector<int> par, order, tin, dep, siz;
    KidsCSR kids;
    vector<ll> down;                    // total edge length from the root; empty when unweighted
    mutable vector<vector<int>> sp;     // Euler-order sparse table, built on the first lca() call
    mutable vector<vector<int>> jump;   // jump[k][v]: 2^k steps up, built on the first ancestor() call
    RootedTree() {}
    void build_meta(int N) {
        int n = (int)order.size();
        siz.assign(N, 1);
        for (int i = n - 1; i > 0; --i) siz[par[order[i]]] += siz[order[i]];
        kids.head.assign(N + 1, 0);
        for (int i = 1; i < n; ++i) ++kids.head[par[order[i]] + 1];
        for (int i = 0; i < N; ++i) kids.head[i + 1] += kids.head[i];
        kids.to.resize(max(0, n - 1));
        vector<int> cur(kids.head.begin(), kids.head.begin() + N);
        for (int i = 1; i < n; ++i) { int u = order[i]; kids.to[cur[par[u]]++] = u; }
    }
    RootedTree(const vector<vector<int>>& adj, int root) : par(adj.size(), -1), tin(adj.size(), -1), dep(adj.size(), 0) {
        int N = (int)adj.size();
        order.reserve(N);
        vector<pair<int, int>> st; st.reserve(N);
        st.push_back({root, 0});
        tin[root] = 0; order.push_back(root);
        while (!st.empty()) {
            int u = st.back().first, i = st.back().second;
            if (i == (int)adj[u].size()) { st.pop_back(); continue; }
            st.back().second++;
            int v = adj[u][i];
            if (tin[v] < 0) { par[v] = u; dep[v] = dep[u] + 1; tin[v] = (int)order.size(); order.push_back(v); st.push_back({v, 0}); }
        }
        build_meta(N);
    }
    // Flat CSR constructor for unweighted undirected edge lists: avoids building vector<vector<int>>
    RootedTree(int N, const vector<int>& head, const vector<int>& to, int root) : par(N, -1), tin(N, -1), dep(N, 0) {
        order.reserve(N);
        vector<pair<int, int>> st; st.reserve(N);
        st.push_back({root, head[root]});
        tin[root] = 0; order.push_back(root);
        while (!st.empty()) {
            int u = st.back().first, i = st.back().second;
            if (i == head[u + 1]) { st.pop_back(); continue; }
            st.back().second = i + 1;
            int v = to[i];
            if (tin[v] < 0) { par[v] = u; dep[v] = dep[u] + 1; tin[v] = (int)order.size(); order.push_back(v); st.push_back({v, head[v]}); }
        }
        build_meta(N);
    }
    // Flat CSR constructor for weighted graphs: single DFS, no plain() copy
    RootedTree(int N, const vector<int>& head, const vector<int>& to, const vector<ll>& wt, int root)
        : par(N, -1), tin(N, -1), dep(N, 0), down(N, 0) {
        order.reserve(N);
        vector<pair<int, int>> st; st.reserve(N);
        st.push_back({root, head[root]});
        tin[root] = 0; order.push_back(root);
        while (!st.empty()) {
            int u = st.back().first, i = st.back().second;
            if (i == head[u + 1]) { st.pop_back(); continue; }
            st.back().second = i + 1;
            int v = to[i];
            if (tin[v] < 0) { par[v] = u; dep[v] = dep[u] + 1; down[v] = down[u] + wt[i]; tin[v] = (int)order.size(); order.push_back(v); st.push_back({v, head[v]}); }
        }
        build_meta(N);
    }
    // Weighted adjacency lists: single DFS directly over arcs without plain() copy
    RootedTree(const vector<vector<pair<int, ll>>>& arcs, int root)
        : par(arcs.size(), -1), tin(arcs.size(), -1), dep(arcs.size(), 0), down(arcs.size(), 0) {
        int N = (int)arcs.size();
        order.reserve(N);
        vector<pair<int, int>> st; st.reserve(N);
        st.push_back({root, 0});
        tin[root] = 0; order.push_back(root);
        while (!st.empty()) {
            int u = st.back().first, i = st.back().second;
            if (i == (int)arcs[u].size()) { st.pop_back(); continue; }
            st.back().second++;
            int v = arcs[u][i].first;
            if (tin[v] < 0) { par[v] = u; dep[v] = dep[u] + 1; down[v] = down[u] + arcs[u][i].second; tin[v] = (int)order.size(); order.push_back(v); st.push_back({v, 0}); }
        }
        build_meta(N);
    }
    ll length(int u, int v) const { return down.empty() ? (ll)dist(u, v) : down[u] + down[v] - 2 * down[lca(u, v)]; }
    int lca(int u, int v) const {
        if (u == v) return u;
        if (sp.empty()) {
            int n = (int)order.size();
            sp.assign(1, vector<int>(n));
            for (int i = 0; i < n; ++i) sp[0][i] = par[order[i]] < 0 ? 0 : tin[par[order[i]]];
            for (int k = 1; (1 << k) <= n; ++k) {
                sp.emplace_back(n - (1 << k) + 1);
                for (int i = 0; i + (1 << k) <= n; ++i) sp[k][i] = min(sp[k - 1][i], sp[k - 1][i + (1 << (k - 1))]);
            }
        }
        int l = min(tin[u], tin[v]) + 1, r = max(tin[u], tin[v]) + 1, k = 31 - __builtin_clz(r - l);
        return order[min(sp[k][l], sp[k][r - (1 << k)])];
    }
    // Heavy-light layout, built on first use: hpos is a preorder that visits the largest child first, so
    // a subtree is one range of positions and a path is O(log n) ranges (one per heavy chain it crosses).
    mutable vector<int> hpos, top;
    void layout() const {
        if (!hpos.empty()) return;
        hpos.assign(par.size(), 0); top.assign(par.size(), 0);
        vector<int> st{order[0]}; top[order[0]] = order[0]; int t = 0;
        while (!st.empty()) {
            int u = st.back(); st.pop_back();
            hpos[u] = t++;
            int h = -1;
            for (int c : kids[u]) if (h < 0 || siz[c] > siz[h]) h = c;
            for (int c : kids[u]) if (c != h) { top[c] = c; st.push_back(c); }
            if (h >= 0) { top[h] = top[u]; st.push_back(h); }
        }
    }
    template <class P> P sub(int v) const { layout(); return P{hpos[v], hpos[v] + siz[v] - 1}; }
    template <class P> ChainList<P> chains(int a, int b) const {
        layout();
        ChainList<P> out;
        while (top[a] != top[b]) {
            if (dep[top[a]] < dep[top[b]]) swap(a, b);
            out.push_back(P{hpos[top[a]], hpos[a]});
            a = par[top[a]];
        }
        out.push_back(P{min(hpos[a], hpos[b]), max(hpos[a], hpos[b])});
        return out;
    }
    bool is_anc(int a, int b) const { return tin[a] <= tin[b] && tin[b] < tin[a] + siz[a]; }
    int dist(int a, int b) const { return dep[a] + dep[b] - 2 * dep[lca(a, b)]; }
    int ancestor(int v, ll k) const {       // -1 above the root
        if (k < 0 || k > dep[v]) return -1;
        if (jump.empty()) {
            jump.push_back(par);
            for (int j = 1; (1 << j) < (int)order.size(); ++j) {
                jump.emplace_back(par.size(), -1);
                for (int x : order) { int h = jump[j - 1][x]; jump[j][x] = h < 0 ? -1 : jump[j - 1][h]; }
            }
        }
        for (int j = 0; k; ++j, k >>= 1) if (k & 1) v = jump[j][v];
        return v;
    }
    int step(int a, int b, ll k) const {    // -1 past b
        int c = lca(a, b); ll up = dep[a] - dep[c], down = dep[b] - dep[c];
        if (k < 0 || k > up + down) return -1;
        return k <= up ? ancestor(a, k) : ancestor(b, up + down - k);
    }
};
// @end

static int cp_fn_iters = 1000;

// @part DenseSet
// Set over a small known key range: one bit per key
struct DenseSet {
    ll lo = 0, cnt = 0;
    vector<uint64_t> w;
    DenseSet() {}
    DenseSet(ll lo_, ll n) : lo(lo_), w((size_t)((n + 63) / 64)) {}
    struct Ins { bool second; };
    bool in_range(ll x) const { return x >= 0 && x < (ll)w.size() * 64; }
    Ins insert(ll x) {
        x -= lo;
        if (!in_range(x)) cp_fail("set key outside its declared range", 0);
        uint64_t& b = w[x >> 6]; uint64_t m = 1ULL << (x & 63);
        if (b & m) return {false};
        b |= m; ++cnt; return {true};
    }
    size_t erase(ll x) {
        x -= lo;
        if (!in_range(x)) return 0;
        uint64_t& b = w[x >> 6]; uint64_t m = 1ULL << (x & 63);
        if (!(b & m)) return 0;
        b &= ~m; --cnt; return 1;
    }
    size_t count(ll x) const { x -= lo; return in_range(x) ? (w[x >> 6] >> (x & 63)) & 1 : 0; }
    size_t size() const { return (size_t)cnt; }
    bool empty() const { return cnt == 0; }
    void clear() { fill(w.begin(), w.end(), 0); cnt = 0; }
    ll next_from(ll i) const {
        ll n = (ll)w.size() * 64;
        while (i < n) { uint64_t b = w[i >> 6] >> (i & 63); if (b) return i + __builtin_ctzll(b); i = (i | 63) + 1; }
        return n;
    }
    ll prev_from(ll i) const {
        while (i >= 0) { uint64_t b = w[i >> 6] << (63 - (i & 63)); if (b) return i - __builtin_clzll(b); i = (i & ~63LL) - 1; }
        return -1;
    }
    struct It {
        const DenseSet* s; ll i;
        ll operator*() const { return s->lo + i; }
        It& operator++() { i = s->next_from(i + 1); return *this; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    It begin() const { return {this, next_from(0)}; }
    It end() const { return {this, (ll)w.size() * 64}; }
    struct RIt { ll v; ll operator*() const { return v; } };
    RIt rbegin() const { return {lo + prev_from((ll)w.size() * 64 - 1)}; }
    bool operator==(const DenseSet& o) const { return lo == o.lo && w == o.w; }
};

// @end
// @part Mask64
// Set over a domain of at most 64 elements: one machine word
struct Mask64 {
    uint64_t m = 0;
    struct Ins { bool second; };
    Ins insert(ll x) { uint64_t b = 1ULL << x; bool n = !(m & b); m |= b; return {n}; }
    size_t erase(ll x) { uint64_t b = 1ULL << x; bool h = m & b; m &= ~b; return h; }
    size_t count(ll x) const { return x >= 0 && x < 64 ? (m >> x) & 1 : 0; }
    size_t size() const { return __builtin_popcountll(m); }
    bool empty() const { return m == 0; }
    void clear() { m = 0; }
    struct It {
        uint64_t r;
        ll operator*() const { return __builtin_ctzll(r); }
        It& operator++() { r &= r - 1; return *this; }
        bool operator!=(const It& o) const { return r != o.r; }
    };
    It begin() const { return {m}; }
    It end() const { return {0}; }
    struct RIt { ll v; ll operator*() const { return v; } };
    RIt rbegin() const { return {63 - __builtin_clzll(m)}; }
    bool operator==(const Mask64& o) const { return m == o.m; }
    bool operator<(const Mask64& o) const { return m < o.m; }
};
// @end
// @part DenseMap
// Map keyed by a domain: one slot per key
template <class V> struct DenseMap {
    mutable vector<V> val;
    vector<char> has;
    ll cnt = 0;
    DenseMap() {}
    explicit DenseMap(ll n) : val((size_t)n), has((size_t)n) {}
    V& operator[](ll k) { if (!has[k]) { has[k] = 1; ++cnt; } return val[k]; }
    struct Ent { ll first; V& second; };
    struct Arrow { Ent e; Ent* operator->() { return &e; } };
    struct It {
        const DenseMap* m; ll i;
        Ent operator*() const { return {i, m->val[i]}; }
        Arrow operator->() const { return {{i, m->val[i]}}; }
        It& operator++() { i = m->next_from(i + 1); return *this; }
        bool operator==(const It& o) const { return i == o.i; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    ll next_from(ll i) const { while (i < (ll)has.size() && !has[i]) ++i; return i; }
    It begin() const { return {this, next_from(0)}; }
    It end() const { return {this, (ll)has.size()}; }
    It find(ll k) const { return k >= 0 && k < (ll)has.size() && has[k] ? It{this, k} : end(); }
    size_t count(ll k) const { return k >= 0 && k < (ll)has.size() && has[k]; }
    size_t erase(ll k) { if (!count(k)) return 0; has[k] = 0; val[k] = V(); --cnt; return 1; }
    size_t size() const { return (size_t)cnt; }
    bool empty() const { return cnt == 0; }
    void clear() { fill(has.begin(), has.end(), 0); fill(val.begin(), val.end(), V()); cnt = 0; }
};
// @end
// @part CpArrHash|CpHash|CpHashMap
// Per-process runtime salt mixed into splitmix64 so fixed-seed anti-hash collision tests cannot target buckets.
static inline uint64_t cp_splitmix64(uint64_t z) {
    z += 0x9e3779b97f4a7c15ULL;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}
static inline uint64_t cp_hash_salt() {
    static const uint64_t s = []() -> uint64_t {
        uint64_t t = (uint64_t)chrono::steady_clock::now().time_since_epoch().count();
        uint64_t a = (uint64_t)(uintptr_t)&t ^ ((uint64_t)getpid() << 32);
        return cp_splitmix64(t ^ cp_splitmix64(a));
    }();
    return s;
}
struct CpArrHash {
    template <size_t K> size_t operator()(const array<ll, K>& a) const {
        uint64_t h = cp_hash_salt();
        for (ll x : a) h = cp_splitmix64((uint64_t)x + h);
        return (size_t)h;
    }
};
struct CpHash {
    size_t operator()(ll x) const { return (size_t)cp_splitmix64((uint64_t)x + cp_hash_salt()); }
    size_t operator()(const string& s) const {
        uint64_t h = cp_hash_salt() ^ 0x9e3779b97f4a7c15ULL;
        for (unsigned char c : s) h = cp_splitmix64(h ^ (uint64_t)c);
        return (size_t)h;
    }
    // a record hashes its fields in order (each record struct has `cp_hash`)
    template <class T> auto operator()(const T& x) const -> decltype(x.cp_hash(*this)) { return x.cp_hash(*this); }
};
// Compact open-addressed hash map: single allocation, linear probing with salted hash and
// backward-shift deletion (no tombstones). For ll keys, packs pair<ll, V> contiguously (16 B/slot,
// single cache line per probe) using LLONG_MIN as the empty sentinel plus a dedicated side slot.
template <class K, class V, class H = CpHash> struct CpHashMap {
    struct Slot { pair<K, V> kv; uint8_t used; };
    char* buf = nullptr;
    uint32_t cap = 0, cnt = 0;
    CpHashMap() {}
    ~CpHashMap() { destroy(); }
    CpHashMap(const CpHashMap& o) {
        if (!o.cap) return;
        alloc(o.cap);
        auto *d = slots(), *s = o.slots();
        for (uint32_t i = 0; i < cap; ++i) if (s[i].used) { ::new (&d[i].kv) pair<K, V>(s[i].kv); d[i].used = 1; }
        cnt = o.cnt;
    }
    CpHashMap(CpHashMap&& o) noexcept : buf(o.buf), cap(o.cap), cnt(o.cnt) { o.buf = nullptr; o.cap = o.cnt = 0; }
    CpHashMap& operator=(const CpHashMap& o) {
        if (this != &o) { CpHashMap tmp(o); *this = std::move(tmp); }
        return *this;
    }
    CpHashMap& operator=(CpHashMap&& o) noexcept {
        if (this != &o) { destroy(); buf = o.buf; cap = o.cap; cnt = o.cnt; o.buf = nullptr; o.cap = o.cnt = 0; }
        return *this;
    }
    Slot* slots() const { return (Slot*)buf; }
    void alloc(uint32_t ncap) {
        buf = (char*)::operator new((size_t)ncap * sizeof(Slot));
        cap = ncap; cnt = 0;
        auto* d = slots();
        for (uint32_t i = 0; i < ncap; ++i) d[i].used = 0;
    }
    void destroy() {
        if (!buf) return;
        auto* s = slots();
        if constexpr (!is_trivially_destructible_v<pair<K, V>>)
            for (uint32_t i = 0; i < cap; ++i) if (s[i].used) s[i].kv.~pair<K, V>();
        ::operator delete(buf);
        buf = nullptr; cap = cnt = 0;
    }
    void rehash(uint32_t ncap) {
        char* obuf = buf; uint32_t ocap = cap;
        auto* os = slots();
        alloc(ncap);
        uint32_t m = cap - 1; auto* ns = slots();
        for (uint32_t i = 0; i < ocap; ++i) if (os[i].used) {
            uint32_t j = (uint32_t)H{}(os[i].kv.first) & m;
            while (ns[j].used) j = (j + 1) & m;
            ::new (&ns[j].kv) pair<K, V>(std::move(os[i].kv));
            ns[j].used = 1; ++cnt;
            if constexpr (!is_trivially_destructible_v<pair<K, V>>) os[i].kv.~pair<K, V>();
        }
        if (obuf) ::operator delete(obuf);
    }
    void reserve_hint(size_t n) {
        if (n <= 8) return;
        uint32_t nc = 8;
        while ((size_t)nc < n && nc < (1U << 30)) nc <<= 1;
        if (nc > cap) rehash(nc);
    }
    void ensure_space() {
        if (!cap) alloc(8);
        else if ((size_t)(cnt + 1) * 10 >= (size_t)cap * 7) rehash(cap << 1);
    }
    uint32_t find_slot(const K& k) const {
        if (!cap) return 0;
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        auto* s = slots();
        while (s[i].used) { if (s[i].kv.first == k) return i; i = (i + 1) & m; }
        return cap;
    }
    V& operator[](const K& k) {
        ensure_space();
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        auto* s = slots();
        while (s[i].used) { if (s[i].kv.first == k) return s[i].kv.second; i = (i + 1) & m; }
        ::new (&s[i].kv) pair<K, V>(k, V());
        s[i].used = 1; ++cnt;
        return s[i].kv.second;
    }
    template <class U> void emplace(const K& k, U&& v) {
        ensure_space();
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        auto* s = slots();
        while (s[i].used) { if (s[i].kv.first == k) return; i = (i + 1) & m; }
        ::new (&s[i].kv) pair<K, V>(k, std::forward<U>(v));
        s[i].used = 1; ++cnt;
    }
    void erase_at(uint32_t i) {
        uint32_t m = cap - 1; auto* s = slots();
        --cnt;
        for (;;) {
            uint32_t j = (i + 1) & m;
            while (s[j].used) {
                uint32_t h = (uint32_t)H{}(s[j].kv.first) & m;
                if (((j - h) & m) >= ((j - i) & m)) break;
                j = (j + 1) & m;
            }
            if (!s[j].used) {
                if constexpr (!is_trivially_destructible_v<pair<K, V>>) s[i].kv.~pair<K, V>();
                s[i].used = 0; return;
            }
            s[i].kv = std::move(s[j].kv);
            i = j;
        }
    }
    struct It {
        const CpHashMap* m; uint32_t i;
        pair<K, V>& operator*() const { return m->slots()[i].kv; }
        pair<K, V>* operator->() const { return &m->slots()[i].kv; }
        It& operator++() { i = m->next_from(i + 1); return *this; }
        bool operator==(const It& o) const { return i == o.i; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    uint32_t next_from(uint32_t i) const {
        auto* s = slots();
        while (i < cap && !s[i].used) ++i;
        return i;
    }
    It begin() const { return {this, cap ? next_from(0) : 0}; }
    It end() const { return {this, cap}; }
    It find(const K& k) const { return {this, find_slot(k)}; }
    size_t count(const K& k) const { return find_slot(k) != cap; }
    size_t erase(const K& k) { uint32_t i = find_slot(k); if (i == cap) return 0; erase_at(i); return 1; }
    void erase(It it) { if (it.i < cap) erase_at(it.i); }
    size_t size() const { return cnt; }
    bool empty() const { return cnt == 0; }
    void clear() {
        if (!buf) return;
        auto* s = slots();
        for (uint32_t i = 0; i < cap; ++i) if (s[i].used) {
            if constexpr (!is_trivially_destructible_v<pair<K, V>>) s[i].kv.~pair<K, V>();
            s[i].used = 0;
        }
        cnt = 0;
    }
    bool operator==(const CpHashMap& o) const {
        if (cnt != o.cnt) return false;
        for (const auto& e : *this) { auto it = o.find(e.first); if (it == o.end() || !(it->second == e.second)) return false; }
        return true;
    }
};
// Specialized sentinel-key table for ll keys: 16 B/slot for ll->ll, one cache line per lookup.
template <class V, class H> struct CpHashMap<ll, V, H> {
    static constexpr ll EMPTY = LLONG_MIN;
    pair<ll, V>* tbl = nullptr;
    uint32_t cap = 0, cnt = 0;
    bool has_e = false;
    pair<ll, V> e_kv{EMPTY, V()};
    CpHashMap() {}
    ~CpHashMap() { destroy(); }
    CpHashMap(const CpHashMap& o) : has_e(o.has_e), e_kv(o.e_kv) {
        if (!o.cap) return;
        alloc(o.cap);
        for (uint32_t i = 0; i < cap; ++i) if (o.tbl[i].first != EMPTY) { ::new (tbl + i) pair<ll, V>(o.tbl[i]); }
        cnt = o.cnt;
    }
    CpHashMap(CpHashMap&& o) noexcept : tbl(o.tbl), cap(o.cap), cnt(o.cnt), has_e(o.has_e), e_kv(std::move(o.e_kv)) {
        o.tbl = nullptr; o.cap = o.cnt = 0; o.has_e = false;
    }
    CpHashMap& operator=(const CpHashMap& o) {
        if (this != &o) { CpHashMap tmp(o); *this = std::move(tmp); }
        return *this;
    }
    CpHashMap& operator=(CpHashMap&& o) noexcept {
        if (this != &o) {
            destroy(); tbl = o.tbl; cap = o.cap; cnt = o.cnt; has_e = o.has_e; e_kv = std::move(o.e_kv);
            o.tbl = nullptr; o.cap = o.cnt = 0; o.has_e = false;
        }
        return *this;
    }
    void alloc(uint32_t ncap) {
        tbl = (pair<ll, V>*)::operator new((size_t)ncap * sizeof(pair<ll, V>));
        cap = ncap; cnt = 0;
        for (uint32_t i = 0; i < ncap; ++i) tbl[i].first = EMPTY;
    }
    void destroy() {
        if (!tbl) return;
        if constexpr (!is_trivially_destructible_v<V>)
            for (uint32_t i = 0; i < cap; ++i) if (tbl[i].first != EMPTY) tbl[i].second.~V();
        ::operator delete(tbl);
        tbl = nullptr; cap = cnt = 0;
    }
    void rehash(uint32_t ncap) {
        pair<ll, V>* ot = tbl; uint32_t oc = cap;
        alloc(ncap);
        uint32_t m = cap - 1;
        for (uint32_t i = 0; i < oc; ++i) if (ot[i].first != EMPTY) {
            uint32_t j = (uint32_t)H{}(ot[i].first) & m;
            while (tbl[j].first != EMPTY) j = (j + 1) & m;
            ::new (tbl + j) pair<ll, V>(std::move(ot[i]));
            ++cnt;
            if constexpr (!is_trivially_destructible_v<V>) ot[i].second.~V();
        }
        if (ot) ::operator delete(ot);
    }
    void reserve_hint(size_t n) {
        if (n <= 8) return;
        uint32_t nc = 8;
        while ((size_t)nc < n && nc < (1U << 30)) nc <<= 1;
        if (nc > cap) rehash(nc);
    }
    void ensure_space() {
        if (!cap) alloc(8);
        else if ((size_t)(cnt + 1) * 10 >= (size_t)cap * 7) rehash(cap << 1);
    }
    uint32_t find_slot(ll k) const {
        if (k == EMPTY) return has_e ? cap : cap + 1;
        if (!cap) return cap + 1;
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        while (tbl[i].first != EMPTY) { if (tbl[i].first == k) return i; i = (i + 1) & m; }
        return cap + 1;
    }
    V& operator[](ll k) {
        if (k == EMPTY) { if (!has_e) { has_e = true; e_kv.second = V(); } return e_kv.second; }
        ensure_space();
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        while (tbl[i].first != EMPTY) { if (tbl[i].first == k) return tbl[i].second; i = (i + 1) & m; }
        ::new (tbl + i) pair<ll, V>(k, V());
        ++cnt;
        return tbl[i].second;
    }
    template <class U> void emplace(ll k, U&& v) {
        if (k == EMPTY) { if (!has_e) { has_e = true; e_kv.second = std::forward<U>(v); } return; }
        ensure_space();
        uint32_t m = cap - 1, i = (uint32_t)H{}(k) & m;
        while (tbl[i].first != EMPTY) { if (tbl[i].first == k) return; i = (i + 1) & m; }
        ::new (tbl + i) pair<ll, V>(k, std::forward<U>(v));
        ++cnt;
    }
    void erase_at(uint32_t i) {
        if (i == cap) { has_e = false; e_kv.second = V(); return; }
        uint32_t m = cap - 1;
        --cnt;
        for (;;) {
            uint32_t j = (i + 1) & m;
            while (tbl[j].first != EMPTY) {
                uint32_t h = (uint32_t)H{}(tbl[j].first) & m;
                if (((j - h) & m) >= ((j - i) & m)) break;
                j = (j + 1) & m;
            }
            if (tbl[j].first == EMPTY) {
                if constexpr (!is_trivially_destructible_v<V>) tbl[i].second.~V();
                tbl[i].first = EMPTY; return;
            }
            tbl[i] = std::move(tbl[j]);
            i = j;
        }
    }
    struct It {
        const CpHashMap* m; uint32_t i;
        pair<ll, V>& operator*() const { return i == m->cap ? const_cast<pair<ll, V>&>(m->e_kv) : m->tbl[i]; }
        pair<ll, V>* operator->() const { return i == m->cap ? const_cast<pair<ll, V>*>(&m->e_kv) : m->tbl + i; }
        It& operator++() { i = m->next_from(i + 1); return *this; }
        bool operator==(const It& o) const { return i == o.i; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    uint32_t next_from(uint32_t i) const {
        while (i < cap && tbl[i].first == EMPTY) ++i;
        if (i == cap && !has_e) return cap + 1;
        return i;
    }
    It begin() const { return {this, next_from(0)}; }
    It end() const { return {this, cap + 1}; }
    It find(ll k) const { return {this, find_slot(k)}; }
    size_t count(ll k) const { return find_slot(k) != cap + 1; }
    size_t erase(ll k) { uint32_t i = find_slot(k); if (i == cap + 1) return 0; erase_at(i); return 1; }
    void erase(It it) { if (it.i <= cap) erase_at(it.i); }
    size_t size() const { return cnt + (has_e ? 1 : 0); }
    bool empty() const { return cnt == 0 && !has_e; }
    void clear() {
        has_e = false; e_kv.second = V();
        if (!tbl) return;
        for (uint32_t i = 0; i < cap; ++i) if (tbl[i].first != EMPTY) {
            if constexpr (!is_trivially_destructible_v<V>) tbl[i].second.~V();
            tbl[i].first = EMPTY;
        }
        cnt = 0;
    }
    bool operator==(const CpHashMap& o) const {
        if (size() != o.size()) return false;
        for (const auto& e : *this) { auto it = o.find(e.first); if (it == o.end() || !(it->second == e.second)) return false; }
        return true;
    }
};
// @end
// @part CpBitset
struct CpBitset {
    int n = 0; vector<uint64_t> w;
    CpBitset() {}
    explicit CpBitset(ll n_) : n((int)n_), w(((size_t)n_ + 63) / 64, 0) {}
    void mask_tail() { int rem = n & 63; if (rem && !w.empty()) w.back() &= (1ULL << rem) - 1; }
    bool add(int i) { uint64_t m = 1ULL << (i & 63); bool r = !(w[i >> 6] & m); w[i >> 6] |= m; return r; }
    bool remove(int i) { uint64_t m = 1ULL << (i & 63); bool r = !!(w[i >> 6] & m); w[i >> 6] &= ~m; return r; }
    void toggle(int i) { w[i >> 6] ^= 1ULL << (i & 63); }
    bool has(int i) const { return (w[i >> 6] >> (i & 63)) & 1; }
    ll len() const { ll c = 0; for (uint64_t x : w) c += __builtin_popcountll(x); return c; }
    bool empty() const { for (uint64_t x : w) if (x) return false; return true; }
    void clear() { fill(w.begin(), w.end(), 0); }
    CpBitset shifted(ll k) const {
        CpBitset r(n);
        if (k >= n || k <= -n) return r;
        if (k == 0) return *this;
        if (k > 0) {
            int ws = (int)(k >> 6), bs = (int)(k & 63);
            for (int i = (int)w.size() - 1; i >= ws; --i) {
                uint64_t v = w[i - ws] << bs;
                if (bs && i - ws - 1 >= 0) v |= w[i - ws - 1] >> (64 - bs);
                r.w[i] = v;
            }
            r.mask_tail();
        } else {
            k = -k;
            int ws = (int)(k >> 6), bs = (int)(k & 63);
            for (int i = 0; i + ws < (int)w.size(); ++i) {
                uint64_t v = w[i + ws] >> bs;
                if (bs && i + ws + 1 < (int)w.size()) v |= w[i + ws + 1] << (64 - bs);
                r.w[i] = v;
            }
        }
        return r;
    }
    CpBitset operator|(const CpBitset& o) const { CpBitset r(n); for (size_t i = 0; i < w.size(); ++i) r.w[i] = w[i] | o.w[i]; return r; }
    CpBitset operator&(const CpBitset& o) const { CpBitset r(n); for (size_t i = 0; i < w.size(); ++i) r.w[i] = w[i] & o.w[i]; return r; }
    CpBitset operator-(const CpBitset& o) const { CpBitset r(n); for (size_t i = 0; i < w.size(); ++i) r.w[i] = w[i] & ~o.w[i]; return r; }
    CpBitset operator^(const CpBitset& o) const { CpBitset r(n); for (size_t i = 0; i < w.size(); ++i) r.w[i] = w[i] ^ o.w[i]; return r; }
};
// @end
// @part cp_set_union|cp_set_inter|cp_set_diff|cp_set_symdiff|cp_set_rank|cp_set_nth
template <class S> static inline S cp_set_union(const S& a, const S& b) { S r = a; r.insert(b.begin(), b.end()); return r; }
template <class S> static inline S cp_set_inter(const S& a, const S& b) { S r; for (const auto& x : a) if (b.count(x)) r.insert(x); return r; }
template <class S> static inline S cp_set_diff(const S& a, const S& b) { S r; for (const auto& x : a) if (!b.count(x)) r.insert(x); return r; }
template <class S> static inline S cp_set_symdiff(const S& a, const S& b) { S r; for (const auto& x : a) if (!b.count(x)) r.insert(x); for (const auto& x : b) if (!a.count(x)) r.insert(x); return r; }
template <class S, class T> static inline ll cp_set_rank(const S& s, const T& x) { return (ll)distance(s.begin(), s.lower_bound(x)); }
template <class S, class O> static inline O cp_set_nth(const S& s, ll k) {
    if (k < 0 || k >= (ll)s.size()) return O();
    auto it = s.begin(); advance(it, k); return O(*it);
}
// @end
// @part CpPt|cp_turn|cp_convex_hull
struct CpPt {
    ll x = 0, y = 0;
    CpPt() {}
    CpPt(ll x_, ll y_) : x(x_), y(y_) {}
    CpPt operator+(CpPt o) const { return {x + o.x, y + o.y}; }
    CpPt operator-(CpPt o) const { return {x - o.x, y - o.y}; }
    CpPt operator-() const { return {-x, -y}; }
    bool operator==(CpPt o) const { return x == o.x && y == o.y; }
    bool operator!=(CpPt o) const { return !(*this == o); }
    bool operator<(CpPt o) const { return x != o.x ? x < o.x : y < o.y; }
    ll dot(CpPt o) const { return x * o.x + y * o.y; }
    ll cross(CpPt o) const { return x * o.y - y * o.x; }
    ll norm2() const { return x * x + y * y; }
};
static inline ll cp_turn(CpPt a, CpPt b, CpPt c) {
    ll cr = (b - a).cross(c - a);
    return cr > 0 ? 1 : cr < 0 ? -1 : 0;
}
static inline vector<CpPt> cp_convex_hull(vector<CpPt> pts) {
    sort(pts.begin(), pts.end());
    pts.erase(unique(pts.begin(), pts.end()), pts.end());
    if (pts.size() <= 1) return pts;
    vector<CpPt> h;
    for (int phase = 0; phase < 2; ++phase) {
        size_t start = h.size();
        for (const auto& p : pts) {
            while (h.size() >= start + 2 && (h[h.size() - 1] - h[h.size() - 2]).cross(p - h[h.size() - 2]) <= 0)
                h.pop_back();
            h.push_back(p);
        }
        h.pop_back();
        reverse(pts.begin(), pts.end());
    }
    return h;
}
// @end
// @part DynMod
template <int Tag> struct DynMod {
    static inline ll M = 1000000007LL;
    ll v = 0;
    DynMod() {}
    DynMod(ll x) { x %= M; if (x < 0) x += M; v = x; }
    static DynMod raw(ll x) { DynMod r; r.v = x; return r; }
    DynMod operator+(DynMod o) const { ll r = v + o.v; return raw(r >= M ? r - M : r); }
    DynMod operator-(DynMod o) const { ll r = v - o.v; return raw(r < 0 ? r + M : r); }
    DynMod operator-() const { return raw(v ? M - v : 0); }
    DynMod operator*(DynMod o) const { return raw((ll)((__int128)v * o.v % M)); }
    DynMod pow(ll e) const { DynMod r(1), b = *this; while (e > 0) { if (e & 1) r = r * b; b = b * b; e >>= 1; } return r; }
    DynMod inv() const { return pow(M - 2); }
    DynMod operator/(DynMod o) const { return *this * o.inv(); }
    DynMod& operator+=(DynMod o) { return *this = *this + o; }
    DynMod& operator-=(DynMod o) { return *this = *this - o; }
    DynMod& operator*=(DynMod o) { return *this = *this * o; }
    DynMod& operator/=(DynMod o) { return *this = *this / o; }
    bool operator==(DynMod o) const { return v == o.v; }
    bool operator!=(DynMod o) const { return v != o.v; }
    ll val() const { return v; }
};
// @end
// @part CpMat
template <class T> struct CpMat {
    int r = 0, c = 0; vector<T> a;
    CpMat() {}
    CpMat(int r_, int c_, T init = T()) : r(r_), c(c_), a((size_t)r_ * c_, init) {}
    T& at(int i, int j) { return a[(size_t)i * c + j]; }
    const T& at(int i, int j) const { return a[(size_t)i * c + j]; }
    CpMat operator*(const CpMat& o) const {
        CpMat res(r, o.c, T(0));
        for (int i = 0; i < r; ++i)
            for (int k = 0; k < c; ++k) {
                T aik = at(i, k);
                for (int j = 0; j < o.c; ++j) res.at(i, j) = res.at(i, j) + aik * o.at(k, j);
            }
        return res;
    }
    CpMat pow(ll e) const {
        CpMat res(r, c, T(0)), base = *this;
        for (int i = 0; i < r; ++i) res.at(i, i) = T(1);
        while (e > 0) { if (e & 1) res = res * base; base = base * base; e >>= 1; }
        return res;
    }
};
// @end
// @part cp_isqrt|cp_pow_mod|cp_factor|cp_sieve
static inline ll cp_isqrt(ll n, int line) {
    if (n < 0) cp_fail("isqrt of negative number", line);
    ll r = (ll)sqrt((long double)n);
    while ((r + 1) > 0 && (r + 1) <= n / (r + 1)) ++r;
    while (r > 0 && r > n / r) --r;
    return r;
}
static inline ll cp_pow_mod(ll a, ll e, ll m, int line) {
    if (m <= 0) cp_fail("pow_mod requires m > 0", line);
    if (m == 1) return 0;
    ll r = 1, b = ((a % m) + m) % m;
    while (e > 0) {
        if (e & 1) r = (ll)((__int128)r * b % m);
        b = (ll)((__int128)b * b % m);
        e >>= 1;
    }
    return r;
}
template <class P> static inline vector<P> cp_factor(ll n) {
    vector<P> res;
    for (ll d = 2; d * d <= n; ++d) {
        if (n % d == 0) {
            ll c = 0;
            while (n % d == 0) { n /= d; ++c; }
            res.push_back(P{d, c});
        }
    }
    if (n > 1) res.push_back(P{n, 1});
    return res;
}
static inline vector<ll> cp_sieve(ll n) {
    vector<ll> ps;
    if (n < 2) return ps;
    vector<uint8_t> comp((size_t)n + 1, 0);
    for (ll i = 2; i <= n; ++i) if (!comp[(size_t)i]) {
        ps.push_back(i);
        if (i <= n / i) for (ll j = i * i; j <= n; j += i) comp[(size_t)j] = 1;
    }
    return ps;
}
// @end
// @part cp_sliding_minmax
template <class T> static inline vector<T> cp_sliding_minmax(const vector<T>& a, ll k, bool is_min) {
    vector<T> res;
    ll n = (ll)a.size();
    if (k <= 0 || k > n) return res;
    deque<ll> dq;
    for (ll i = 0; i < n; ++i) {
        while (!dq.empty() && dq.front() <= i - k) dq.pop_front();
        while (!dq.empty() && (is_min ? a[dq.back()] >= a[i] : a[dq.back()] <= a[i])) dq.pop_back();
        dq.push_back(i);
        if (i >= k - 1) res.push_back(a[dq.front()]);
    }
    return res;
}
// @end
// @part cp_perm_inv|cp_perm_cycles
static inline vector<int> cp_perm_inv(const vector<int>& p) {
    vector<int> inv(p.size());
    for (size_t i = 0; i < p.size(); ++i) inv[(size_t)p[i]] = (int)i;
    return inv;
}
static inline vector<vector<int>> cp_perm_cycles(const vector<int>& p) {
    size_t n = p.size();
    vector<uint8_t> vis(n, 0);
    vector<vector<int>> cycs;
    for (size_t i = 0; i < n; ++i) if (!vis[i]) {
        vector<int> c;
        size_t u = i;
        while (!vis[u]) { vis[u] = 1; c.push_back((int)u); u = (size_t)p[u]; }
        cycs.push_back(std::move(c));
    }
    return cycs;
}
// @end
// @part cp_grid_col|cp_grid_flipped|cp_grid_transposed
template <class T> static inline vector<T> cp_grid_col(const vector<T>& g, int rows, int cols, int c) {
    vector<T> res((size_t)rows);
    for (int r = 0; r < rows; ++r) res[(size_t)r] = g[(size_t)r * cols + c];
    return res;
}
template <class T> static inline vector<T> cp_grid_flipped(const vector<T>& g, int rows, int cols) {
    vector<T> res(g.size());
    for (int r = 0; r < rows; ++r)
        for (int c = 0; c < cols; ++c)
            res[(size_t)(rows - 1 - r) * cols + c] = g[(size_t)r * cols + c];
    return res;
}
template <class T> static inline vector<T> cp_grid_transposed(const vector<T>& g, int rows, int cols) {
    vector<T> res(g.size());
    for (int r = 0; r < rows; ++r)
        for (int c = 0; c < cols; ++c)
            res[(size_t)c * rows + r] = g[(size_t)r * cols + c];
    return res;
}
// @end
// @part CpFenwick2D
template <class T> struct CpFenwick2D {
    int r = 0, c = 0; vector<T> t;
    CpFenwick2D() {}
    CpFenwick2D(int r_, int c_) : r(r_), c(c_), t((size_t)(r_ + 1) * (c_ + 1), T(0)) {}
    void add(int i, int j, T d) {
        for (int x = i + 1; x <= r; x += x & -x)
            for (int y = j + 1; y <= c; y += y & -y)
                t[(size_t)x * (c + 1) + y] += d;
    }
    T pre(int i, int j) const {
        T s = T(0);
        for (int x = i; x > 0; x -= x & -x)
            for (int y = j; y > 0; y -= y & -y)
                s += t[(size_t)x * (c + 1) + y];
        return s;
    }
    T sum(Rng rr, Rng cr) const {
        if (rr.lo >= rr.hi || cr.lo >= cr.hi) return T(0);
        return pre((int)rr.hi, (int)cr.hi) - pre((int)rr.lo, (int)cr.hi) - pre((int)rr.hi, (int)cr.lo) + pre((int)rr.lo, (int)cr.lo);
    }
};
// @end
// @part CpFlowGraph
struct CpFlowGraph {
    struct Edge { int to, rev; ll cap; };
    int n = 0; vector<vector<Edge>> g;
    CpFlowGraph() {}
    explicit CpFlowGraph(int n_) : n(n_), g((size_t)n_) {}
    void add_edge(int u, int v, ll cap) {
        g[u].push_back({v, (int)g[v].size(), cap});
        g[v].push_back({u, (int)g[u].size() - 1, 0});
    }
    ll max_flow(int s, int t) {
        ll flow = 0; vector<int> lvl(n), ptr(n);
        while (true) {
            fill(lvl.begin(), lvl.end(), -1);
            queue<int> q; lvl[s] = 0; q.push(s);
            while (!q.empty()) {
                int u = q.front(); q.pop();
                for (auto& e : g[u]) if (e.cap > 0 && lvl[e.to] < 0) { lvl[e.to] = lvl[u] + 1; q.push(e.to); }
            }
            if (lvl[t] < 0) break;
            fill(ptr.begin(), ptr.end(), 0);
            auto dfs = [&](auto& self, int u, ll pushed) -> ll {
                if (u == t || pushed == 0) return pushed;
                for (int& cid = ptr[u]; cid < (int)g[u].size(); ++cid) {
                    auto& e = g[u][cid];
                    if (e.cap > 0 && lvl[e.to] == lvl[u] + 1) {
                        ll tr = self(self, e.to, min(pushed, e.cap));
                        if (tr > 0) { e.cap -= tr; g[e.to][e.rev].cap += tr; return tr; }
                    }
                }
                return 0;
            };
            while (ll p = dfs(dfs, s, LLONG_MAX)) flow += p;
        }
        return flow;
    }
};
// @end
// @part CpTwoSat
struct CpTwoSat {
    int n = 0; vector<vector<int>> g, gr;
    CpTwoSat() {}
    explicit CpTwoSat(int n_) : n(n_), g(2 * n_), gr(2 * n_) {}
    void add_imp(int a, int b) { g[a].push_back(b); gr[b].push_back(a); }
    void or_clause(int u, bool bu, int v, bool bv) {
        int a = 2 * u + (bu ? 1 : 0), b = 2 * v + (bv ? 1 : 0);
        add_imp(a ^ 1, b); add_imp(b ^ 1, a);
    }
    Opt<vector<uint8_t>> solve() const {
        int m = 2 * n; vector<int> ord, comp(m, -1); vector<uint8_t> vis(m, 0);
        auto d1 = [&](auto& self, int u) -> void { vis[u] = 1; for (int v : g[u]) if (!vis[v]) self(self, v); ord.push_back(u); };
        for (int i = 0; i < m; ++i) if (!vis[i]) d1(d1, i);
        int cid = 0;
        auto d2 = [&](auto& self, int u) -> void { comp[u] = cid; for (int v : gr[u]) if (comp[v] < 0) self(self, v); };
        for (int i = m - 1; i >= 0; --i) if (comp[ord[i]] < 0) { d2(d2, ord[i]); ++cid; }
        vector<uint8_t> ans(n, 0);
        for (int i = 0; i < n; ++i) {
            if (comp[2 * i] == comp[2 * i + 1]) return Opt<vector<uint8_t>>();
            ans[i] = comp[2 * i + 1] > comp[2 * i];
        }
        return Opt<vector<uint8_t>>(ans);
    }
};
// @end
// @part cp_topo
static inline Opt<vector<int>> cp_topo(const vector<vector<int>>& adj) {
    int n = (int)adj.size();
    vector<int> indeg(n, 0), ord;
    for (int u = 0; u < n; ++u) for (int v : adj[u]) ++indeg[v];
    queue<int> q;
    for (int u = 0; u < n; ++u) if (indeg[u] == 0) q.push(u);
    while (!q.empty()) {
        int u = q.front(); q.pop(); ord.push_back(u);
        for (int v : adj[u]) if (--indeg[v] == 0) q.push(v);
    }
    return (int)ord.size() == n ? Opt<vector<int>>(ord) : Opt<vector<int>>();
}
// @end
// @part cp_scc_list
static inline vector<vector<int>> cp_scc_list(const vector<vector<int>>& adj) {
    int n = (int)adj.size();
    vector<vector<int>> gr(n), res;
    for (int u = 0; u < n; ++u) for (int v : adj[u]) gr[v].push_back(u);
    vector<uint8_t> vis(n, 0); vector<int> ord;
    auto d1 = [&](auto& self, int u) -> void { vis[u] = 1; for (int v : adj[u]) if (!vis[v]) self(self, v); ord.push_back(u); };
    for (int i = 0; i < n; ++i) if (!vis[i]) d1(d1, i);
    fill(vis.begin(), vis.end(), 0);
    auto d2 = [&](auto& self, int u, vector<int>& c) -> void { vis[u] = 1; c.push_back(u); for (int v : gr[u]) if (!vis[v]) self(self, v, c); };
    for (int i = n - 1; i >= 0; --i) if (!vis[ord[i]]) { vector<int> c; d2(d2, ord[i], c); res.push_back(std::move(c)); }
    return res;
}
// @end
// @part cp_z_function|cp_prefix_function|CpSuffixArray|CpRollingHash|cp_palindromes
static inline vector<ll> cp_z_function(const string& s) {
    int n = (int)s.size(); vector<ll> z(n, 0);
    for (int i = 1, l = 0, r = 0; i < n; ++i) {
        if (i < r) z[i] = min<ll>(r - i, z[i - l]);
        while (i + z[i] < n && s[z[i]] == s[i + z[i]]) ++z[i];
        if (i + z[i] > r) { l = i; r = i + (int)z[i]; }
    }
    return z;
}
static inline vector<ll> cp_prefix_function(const string& s) {
    int n = (int)s.size(); vector<ll> pi(n, 0);
    for (int i = 1; i < n; ++i) {
        ll j = pi[i - 1];
        while (j > 0 && s[i] != s[j]) j = pi[j - 1];
        if (s[i] == s[j]) ++j;
        pi[i] = j;
    }
    return pi;
}
struct CpSuffixArray {
    vector<ll> sa, lcp;
    CpSuffixArray() {}
    explicit CpSuffixArray(const string& s) {
        int n = (int)s.size(); sa.resize(n); iota(sa.begin(), sa.end(), 0);
        vector<int> rk(n), tmp(n);
        for (int i = 0; i < n; ++i) rk[i] = (unsigned char)s[i];
        for (int k = 1; k < n; k <<= 1) {
            auto cmp = [&](ll a, ll b) {
                if (rk[a] != rk[b]) return rk[a] < rk[b];
                int ra = a + k < n ? rk[a + k] : -1, rb = b + k < n ? rk[b + k] : -1;
                return ra < rb;
            };
            sort(sa.begin(), sa.end(), cmp);
            tmp[sa[0]] = 0;
            for (int i = 1; i < n; ++i) tmp[sa[i]] = tmp[sa[i - 1]] + (cmp(sa[i - 1], sa[i]) ? 1 : 0);
            rk.swap(tmp);
            if (rk[sa[n - 1]] == n - 1) break;
        }
        lcp.assign(max(0, n - 1), 0);
        for (int i = 0, h = 0; i < n; ++i) if (rk[i] > 0) {
            int j = (int)sa[rk[i] - 1];
            while (i + h < n && j + h < n && s[i + h] == s[j + h]) ++h;
            lcp[rk[i] - 1] = h;
            if (h > 0) --h;
        }
    }
};
struct CpRollingHash {
    static constexpr uint64_t MOD = (1ULL << 61) - 1;
    static constexpr uint64_t BASE = 1315423911ULL;
    vector<uint64_t> h, pw;
    static inline uint64_t mul(uint64_t a, uint64_t b) {
        __uint128_t t = (__uint128_t)a * b;
        uint64_t r = (uint64_t)(t >> 61) + (uint64_t)(t & MOD);
        return r >= MOD ? r - MOD : r;
    }
    CpRollingHash() {}
    explicit CpRollingHash(const string& s) : h(s.size() + 1, 0), pw(s.size() + 1, 1) {
        for (size_t i = 0; i < s.size(); ++i) {
            pw[i + 1] = mul(pw[i], BASE);
            uint64_t v = mul(h[i], BASE) + (unsigned char)s[i] + 1;
            h[i + 1] = v >= MOD ? v - MOD : v;
        }
    }
    ll get(Rng r) const {
        if (r.lo >= r.hi) return 0;
        uint64_t sub = mul(h[(size_t)r.lo], pw[(size_t)(r.hi - r.lo)]);
        uint64_t v = h[(size_t)r.hi] + MOD - sub;
        return (ll)(v >= MOD ? v - MOD : v);
    }
};
static inline vector<ll> cp_palindromes(const string& s) {
    string t = "#";
    for (char c : s) { t += c; t += '#'; }
    int n = (int)t.size(); vector<ll> p(n, 0);
    for (int i = 0, l = 0, r = -1; i < n; ++i) {
        int k = i > r ? 1 : (int)min<ll>(p[l + r - i], r - i + 1);
        while (0 <= i - k && i + k < n && t[i - k] == t[i + k]) ++k;
        p[i] = k--;
        if (i + k > r) { l = i - k; r = i + k; }
    }
    return p;
}
// @end
// @part CpCustomLazy
struct CpCustomLazy {
    int n = 0; ll id_tag = 0;
    std::function<ll(ll, ll)> comp, app;
    vector<ll> a, lz;
    CpCustomLazy() {}
    CpCustomLazy(int n_, ll id_, std::function<ll(ll, ll)> c_, std::function<ll(ll, ll)> ap_)
        : n(n_), id_tag(id_), comp(c_), app(ap_), a(n_, 0), lz(n_, id_) {}
    void apply(Rng r, ll f) {
        for (ll i = max(0LL, r.lo); i < min((ll)n, r.hi); ++i) a[i] = app(f, a[i]);
    }
    ll fold(Rng r) const {
        ll s = 0;
        for (ll i = max(0LL, r.lo); i < min((ll)n, r.hi); ++i) s += a[i];
        return s;
    }
};
// @end
// @part CpStepsT
template <class K, class V> struct CpStepsT {
    map<K, V> m; V def;
    CpStepsT() : def(V()) {}
    explicit CpStepsT(V d) : def(d) {}
};
// @end
// @part CpDir4|CpDir8
struct CpDir4 {
    int d = 0; // 0=U(-1,0), 1=R(0,1), 2=D(1,0), 3=L(0,-1)
    static constexpr int DR[4] = {-1, 0, 1, 0};
    static constexpr int DC[4] = {0, 1, 0, -1};
    CpDir4() {}
    explicit CpDir4(int d_) : d(d_ & 3) {}
    int dr() const { return DR[d]; }
    int dc() const { return DC[d]; }
    CpDir4 cw() const { return CpDir4((d + 1) & 3); }
    CpDir4 ccw() const { return CpDir4((d + 3) & 3); }
    CpDir4 back() const { return CpDir4((d + 2) & 3); }
    bool operator==(CpDir4 o) const { return d == o.d; }
    bool operator!=(CpDir4 o) const { return d != o.d; }
    static CpDir4 from(const string& order, char ch, int line) {
        static constexpr int MAP[4] = {0, 2, 3, 1}; // default "UDLR" -> U=0, D=2, L=3, R=1
        if (order == "NESW" || order == "URDL") {
            size_t p = order.find(ch);
            if (p != string::npos) return CpDir4((int)p);
        } else if (order.size() == 4) {
            size_t p = order.find(ch);
            if (p != string::npos) return CpDir4(MAP[p]);
        }
        cp_fail("Dir4.from: character not found", line);
        return CpDir4(0);
    }
};
struct CpDir8 {
    int d = 0;
    static constexpr int DR[8] = {-1, -1, 0, 1, 1, 1, 0, -1};
    static constexpr int DC[8] = {0, 1, 1, 1, 0, -1, -1, -1};
    CpDir8() {}
    explicit CpDir8(int d_) : d(d_ & 7) {}
    int dr() const { return DR[d]; }
    int dc() const { return DC[d]; }
    bool operator==(CpDir8 o) const { return d == o.d; }
    bool operator!=(CpDir8 o) const { return d != o.d; }
};
// @end
// @part CpCounter|CpDenseCounter
// Counter<K>: how many of each key, and how many keys have a count that is not zero. Stand-in for the
// library struct, whose `at` is a projection with a write-back (see tests/lend/writeback.cp).
template <class C, class K> struct CpCount {
    C* c; K k;
    operator ll() const { return c->get(k); }
    CpCount& operator=(ll v) { c->set(k, v); return *this; }
    CpCount& operator+=(ll d) { c->inc(k, d); return *this; }
    CpCount& operator-=(ll d) { c->inc(k, -d); return *this; }
};
template <class M> static inline auto cp_try_reserve(M& m, size_t n, int) -> decltype(m.reserve_hint(n), void()) { m.reserve_hint(n); }
template <class M> static inline void cp_try_reserve(M&, size_t, long) {}
template <class M, class K> struct CpCounter {
    M m; ll distinct = 0;
    void reserve_hint(size_t n) { cp_try_reserve(m, n, 0); }
    ll get(const K& k) const { auto it = m.find(k); return it == m.end() ? 0 : it->second; }
    void set(const K& k, ll v) {
        auto it = m.find(k);
        if (it == m.end()) { if (v != 0) { m.emplace(k, v); ++distinct; } }
        else if (v == 0) { m.erase(it); --distinct; }
        else it->second = v;
    }
    void inc(const K& k, ll d = 1) {
        if (d == 0) return;
        ll& v = m[k];
        if (v == 0) ++distinct;
        v += d;
        if (v == 0) { m.erase(k); --distinct; }
    }
    CpCount<CpCounter, K> at(const K& k) { return {this, k}; }
};
struct CpDenseCounter {
    vector<ll> m; ll distinct = 0;
    CpDenseCounter() {}
    explicit CpDenseCounter(ll n) : m(n, 0) {}
    void reserve_hint(size_t) {}
    ll get(int k) const { return m[k]; }
    void set(int k, ll v) { distinct += (v != 0) - (m[k] != 0); m[k] = v; }
    void inc(int k, ll d = 1) { ll nv = m[k] + d; distinct += (nv != 0) - (m[k] != 0); m[k] = nv; }
    CpCount<CpDenseCounter, int> at(int k) { return {this, k}; }
};
// @end
// @part CpPrefix
// prefix(a): p[r][c] is the sum of the cells before boundary r and before boundary c (a 1-D array is one column)
template <class T = ll> struct CpPrefix {
    vector<T> p; ll w = 0;
    CpPrefix() {}
    CpPrefix(ll rows, ll cols) : p((size_t)(rows + 1) * (size_t)(cols + 1), 0), w(cols + 1) {}
    template <class A> CpPrefix(const A& a, ll rows, ll cols) : p((size_t)(rows + 1) * (size_t)(cols + 1), 0), w(cols + 1) {
        for (ll r = 0; r < rows; ++r) {
            ll r0 = r * w, r1 = r0 + w, ar = r * cols;
            for (ll c = 0; c < cols; ++c)
                p[r1 + c + 1] = p[r0 + c + 1] + p[r1 + c] - p[r0 + c] + (T)a[ar + c];
        }
    }
    ll wrap(ll from, ll k, ll n, int line) const {     // 1-D: k cells starting at `from`, going round past the end
        if (k < 0 || n <= 0) cp_fail("sum(from:, len:) needs len >= 0 and a non-empty array", line);
        auto upto = [&](ll b) -> ll { return (ll)p[b * w + 1]; };
        ll rest = k % n, to = from + rest;
        return (k / n) * upto(n) + (to <= n ? upto(to) - upto(from) : upto(n) - upto(from) + upto(to - n));
    }
    ll sum(Rng rs, Rng cs) const {      // an empty range sums to 0
        if (rs.lo >= rs.hi || cs.lo >= cs.hi) return 0;
        return (ll)p[rs.hi * w + cs.hi] - (ll)p[rs.lo * w + cs.hi] - (ll)p[rs.hi * w + cs.lo] + (ll)p[rs.lo * w + cs.lo];
    }
};
// @end
// @part I32
struct I32 {
    int v;
    I32(ll x = 0) : v((int)x) {}
    operator ll() const { return v; }
    I32& operator+=(ll x) { v += (int)x; return *this; }
    I32& operator-=(ll x) { v -= (int)x; return *this; }
};
// @end
// @part cp_bfs
// bfs(adj, from): fewest arcs from any of `from` to each vertex. Prototype stand-in for std.graph.bfs.
static inline int cp_arc_to(int v) { return v; }
template <class A> static inline auto cp_arc_to(const A& a) -> decltype((int)a.to) { return (int)a.to; }
template <class O, class F, class Adj> vector<O> cp_bfs(ll n, const F& from, Adj adj) {
    vector<O> dist((size_t)n); vector<int> q;
    for (int s : from) if (!dist[s].has()) { dist[s] = O(0); q.push_back(s); }
    for (size_t h = 0; h < q.size(); ++h) {
        int u = q[h]; ll d = dist[u].get() + 1;
        for (const auto& a : adj(u)) { int v = cp_arc_to(a); if (!dist[v].has()) { dist[v] = O(d); q.push_back(v); } }
    }
    return dist;
}
// @end
// @part cp_choose
// choose(n, k): the number of k-subsets of n things, exactly; 0 when k is outside 0..=n
static inline ll cp_choose(ll n, ll k, int line) {
    if (k < 0 || k > n) return 0;
    k = min(k, n - k);
    __int128 r = 1;
    for (ll i = 0; i < k; ++i) {
        r = r * (n - i) / (i + 1);
        if (r > (__int128)LLONG_MAX) cp_fail("Int overflow", line);
    }
    return (ll)r;
}
// @end
// @part CpSteps
// Steps<K>: a function on a line that changes at few places. add(range, x) adds x over a range; runs(range)
// reads it back as maximal stretches of one value. Prototype stand-in for std.steps.
struct CpSteps {
    mutable vector<pair<ll, ll>> ev;    // lazy-sorted (coord, delta) events
    mutable bool dirty = false;
    void add(Rng r, ll x) {
        if (r.lo < r.hi && x != 0) {
            ev.emplace_back(r.lo, x);
            ev.emplace_back(r.hi, -x);
            dirty = true;
        }
    }
    void normalize() const {
        if (!dirty) return;
        dirty = false;
        if (ev.empty()) return;
        sort(ev.begin(), ev.end(), [](const pair<ll, ll>& a, const pair<ll, ll>& b) { return a.first < b.first; });
        size_t w = 0;
        for (size_t i = 0; i < ev.size(); ++i) {
            if (w && ev[w - 1].first == ev[i].first) ev[w - 1].second += ev[i].second;
            else ev[w++] = ev[i];
        }
        size_t k = 0;
        for (size_t i = 0; i < w; ++i) if (ev[i].second != 0) ev[k++] = ev[i];
        ev.resize(k);
    }
    template <class Tup> vector<Tup> runs(Rng clip) const {
        normalize();
        vector<Tup> out;
        if (clip.lo >= clip.hi) return out;
        ll cur = 0, at = clip.lo;
        size_t i = 0;
        for (; i < ev.size() && ev[i].first <= clip.lo; ++i) cur += ev[i].second;
        auto put = [&](ll hi) {
            if (hi <= at) return;
            if (!out.empty() && (ll)out.back().f1 == cur) out.back().f0.hi = hi; else out.push_back(Tup{Rng{at, hi}, cur});
            at = hi;
        };
        for (; i < ev.size() && ev[i].first < clip.hi; ++i) { put(ev[i].first); cur += ev[i].second; }
        put(clip.hi);
        return out;
    }
};
// @end
// @part cp_take
// take(place): the value, leaving the place empty
template <class T> static inline T cp_take(T& x, T empty) { T v = std::move(x); x = std::move(empty); return v; }
// @end
