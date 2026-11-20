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
#define CP_ASSERT(c, msg, line) do { if (!(c)) cp_fail(msg, line); } while (0)
#else
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

template <class T> struct Opt {
    T v{}; bool h = false;
    Opt() {}
    Opt(T x) : v(x), h(true) {}
    bool has() const { return h; }
    const T& get() const { return v; }
    bool operator==(const Opt& o) const { return h == o.h && (!h || v == o.v); }
    bool operator!=(const Opt& o) const { return !(*this == o); }
};
template <> struct Opt<int> {
    int v = -1;
    Opt() {}
    Opt(int x) : v(x) {}
    bool has() const { return v >= 0; }
    int get() const { return v; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
template <> struct Opt<bool> {
    signed char v = -1;
    Opt() {}
    Opt(bool x) : v(x) {}
    bool has() const { return v >= 0; }
    bool get() const { return v == 1; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
// ?Int stores none as LLONG_MIN, so it costs no more space than Int
template <> struct Opt<ll> {
    ll v = LLONG_MIN;
    Opt() {}
    Opt(ll x) : v(x) { if (x == LLONG_MIN) cp_fail("?Int cannot hold the minimum Int", 0); }
    bool has() const { return v != LLONG_MIN; }
    ll get() const { return v; }
    bool operator==(const Opt& o) const { return v == o.v; }
    bool operator!=(const Opt& o) const { return v != o.v; }
};
template <class T> static inline T opt_unwrap(const Opt<T>& o, int line) {
    if (!o.has()) cp_fail("unwrapped none", line);
    return o.get();
}
template <class T, class U> static inline void opt_min(Opt<T>& o, U x) { if (!o.has() || T(x) < o.get()) o = Opt<T>(T(x)); }
template <class T, class U> static inline void opt_max(Opt<T>& o, U x) { if (!o.has() || o.get() < T(x)) o = Opt<T>(T(x)); }
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
    bool operator!=(const OptI32& o) const { return v != o.v; }
};
static inline ll opt_unwrap(const OptI32& o, int line) {
    if (!o.has()) cp_fail("unwrapped none", line);
    return o.get();
}
template <class U> static inline void opt_min(OptI32& o, U x) { if (!o.has() || (ll)x < o.get()) o = OptI32((ll)x); }
template <class U> static inline void opt_max(OptI32& o, U x) { if (!o.has() || o.get() < (ll)x) o = OptI32((ll)x); }
// @end

struct Rng {
    ll lo, hi;
    bool operator==(const Rng& o) const { return lo == o.lo && hi == o.hi; }
    bool operator!=(const Rng& o) const { return !(*this == o); }
    bool operator<(const Rng& o) const { return lo != o.lo ? lo < o.lo : hi < o.hi; }
    bool operator>(const Rng& o) const { return o < *this; }
};
static inline Opt<int> cp_step(ll ord, ll k, bool minus, ll len) {
    __int128 x = (__int128)ord + (minus ? -(__int128)k : (__int128)k);
    return x >= 0 && x < len ? Opt<int>((int)x) : Opt<int>();
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
    if (i < lo || i >= hi) cp_fail("position is outside the slice", line);
    return i;
}
template <class T> static inline void cp_swap_view(CpView<T> a, CpView<T> b) { swap_ranges(a.begin(), a.end(), b.begin()); }
// @end

template <ll M> struct Mod {
    ll v;
    Mod(ll x = 0) : v(((x % M) + M) % M) {}
    static Mod raw(ll x) { Mod r; r.v = x; return r; }
    Mod operator+(Mod o) const { ll s = v + o.v; return raw(s >= M ? s - M : s); }
    Mod operator-(Mod o) const { ll s = v - o.v; return raw(s < 0 ? s + M : s); }
    Mod operator*(Mod o) const { return raw((ll)((__int128)v * o.v % M)); }
    Mod operator-() const { return raw(v ? M - v : 0); }
    Mod pow(ll e) const { Mod r(1), b = *this; for (; e; e >>= 1, b = b * b) if (e & 1) r = r * b; return r; }
    Mod inv() const { return pow(M - 2); }
    Mod operator/(Mod o) const { return *this * o.inv(); }
    Mod& operator+=(Mod o) { return *this = *this + o; }
    Mod& operator-=(Mod o) { return *this = *this - o; }
    Mod& operator*=(Mod o) { return *this = *this * o; }
    bool operator==(Mod o) const { return v == o.v; }
    bool operator!=(Mod o) const { return v != o.v; }
};

// ---- input
static char cp_ibuf[1 << 16]; static size_t cp_ipos = 0, cp_ilen = 0;
static inline int cp_getc() {
    if (cp_ipos == cp_ilen) { cp_ilen = (size_t)max<long>(0, read(0, cp_ibuf, sizeof cp_ibuf));  /* read(2), not fread: returns what is there, so `ask` cannot deadlock on a pipe */ cp_ipos = 0; if (!cp_ilen) return -1; }
    return cp_ibuf[cp_ipos++];
}
static inline ll rd_int() {
    int c = cp_getc(); while (c == ' ' || c == '\n' || c == '\r' || c == '\t') c = cp_getc();
    if (c < 0) cp_fail("unexpected end of input", 0);
    bool neg = c == '-'; if (neg) c = cp_getc();
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

// ---- output
static string cp_obuf;
static inline void out_int(ll x) { char b[24]; int n = 0; bool neg = x < 0; unsigned long long u = neg ? -(unsigned long long)x : x; do b[n++] = '0' + u % 10; while (u /= 10); if (neg) cp_obuf += '-'; while (n) cp_obuf += b[--n]; }
static inline void out_str(const string& s) { cp_obuf += s; }
static inline void out_char(char c) { cp_obuf += c; }
// @part cp_ediv|cp_emod
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
// @end
static inline void out_flush() { fwrite(cp_obuf.data(), 1, cp_obuf.size(), stdout); cp_obuf.clear(); }

static inline ll cp_ipow(ll b, ll e) { ll r = 1; while (e--) r *= b; return r; }
static inline size_t cp_lidx(ll i, size_t n, int line) { if (i < 0 || (size_t)i >= n) cp_fail("list index out of range", line); return (size_t)i; }
static inline ll cp_rfirst(Rng r, int line) { if (r.lo >= r.hi) cp_fail("empty range", line); return r.lo; }
static inline ll cp_rlast(Rng r, int line) { if (r.lo >= r.hi) cp_fail("empty range", line); return r.hi - 1; }
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
};

// AddMax: add to an inclusive range, max of an inclusive range. Prototype stand-in for a lazy std.segtree.
// Lazy segment tree: assign to a range, sum of a range (stdlib/lazy.cp LazySegTree with the AssignSum action)
struct AssignSum {
    int n = 0; vector<ll> v, lz;
    AssignSum() {}
    AssignSum(int n_) : n(max(n_, 1)), v(4 * max(n_, 1)), lz(4 * max(n_, 1), LLONG_MIN) {}
    void put(int o, int l, int r, ll x) { v[o] = x * (r - l + 1); lz[o] = x; }
    void push(int o, int l, int m, int r) { if (lz[o] != LLONG_MIN) { put(2 * o, l, m, lz[o]); put(2 * o + 1, m + 1, r, lz[o]); lz[o] = LLONG_MIN; } }
    void assign(int o, int l, int r, int ql, int qr, ll x) {
        if (ql <= l && r <= qr) { put(o, l, r, x); return; }
        int m = (l + r) / 2; push(o, l, m, r);
        if (ql <= m) assign(2 * o, l, m, ql, qr, x);
        if (qr > m) assign(2 * o + 1, m + 1, r, ql, qr, x);
        v[o] = v[2 * o] + v[2 * o + 1];
    }
    ll sum(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) return v[o];
        int m = (l + r) / 2; push(o, l, m, r);
        return (ql <= m ? sum(2 * o, l, m, ql, qr) : 0) + (qr > m ? sum(2 * o + 1, m + 1, r, ql, qr) : 0);
    }
    void assign(int l, int r, ll x) { if (l <= r) assign(1, 0, n - 1, l, r, x); }
    ll sum(int l, int r) { return l <= r ? sum(1, 0, n - 1, l, r) : 0; }
};

struct AddMax {
    int n = 0; vector<ll> v, lz;
    AddMax() {}
    AddMax(int n_) : n(max(n_, 1)), v(4 * max(n_, 1)), lz(4 * max(n_, 1)) {}
    void add(int o, int l, int r, int ql, int qr, ll x) {
        if (ql <= l && r <= qr) { v[o] += x; lz[o] += x; return; }
        int m = (l + r) / 2;
        if (ql <= m) add(2 * o, l, m, ql, qr, x);
        if (m < qr) add(2 * o + 1, m + 1, r, ql, qr, x);
        v[o] = max(v[2 * o], v[2 * o + 1]) + lz[o];
    }
    ll query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) return v[o];
        int m = (l + r) / 2; ll best = LLONG_MIN;
        if (ql <= m) best = max(best, query(2 * o, l, m, ql, qr));
        if (m < qr) best = max(best, query(2 * o + 1, m + 1, r, ql, qr));
        return best + lz[o];
    }
    void add(int l, int r, ll x) { if (l <= r) add(1, 0, n - 1, l, r, x); }
    ll query(int l, int r) const { return query(1, 0, n - 1, l, r); }
    ll top() const { return v[1]; }
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
    CpDsu() {}
    explicit CpDsu(ll n) : p(n), sz(n, 1), parts(n) { iota(p.begin(), p.end(), 0); }
    int find(int v) { while (p[v] != v) v = p[v] = p[p[v]]; return v; }
    bool unite(int a, int b) { a = find(a); b = find(b); if (a == b) return false; if (sz[a] < sz[b]) swap(a, b); p[b] = a; sz[a] += sz[b]; --parts; return true; }
    bool same(int a, int b) { return find(a) == find(b); }
    int size(int v) { return sz[find(v)]; }
};

// RootedTree: the depth-first spanning tree of `adj` from `root`, with parent, preorder, O(1) lca and level
// ancestors. On a tree that is the tree itself; on any other connected graph every edge left out joins a
// vertex to one of its ancestors. Prototype stand-in for std.tree.
struct RootedTree {
    vector<int> par, order, tin, dep, siz; vector<vector<int>> sp, kids;
    vector<ll> down;                    // total edge length from the root; the depth when edges have no lengths
    mutable vector<vector<int>> jump;   // jump[k][v]: 2^k steps up, built on the first ancestor() call
    RootedTree() {}
    RootedTree(const vector<vector<int>>& adj, int root) : par(adj.size(), -1), tin(adj.size(), -1), dep(adj.size(), 0) {
        vector<pair<int, int>> st{{root, 0}};
        tin[root] = 0; order.push_back(root);
        while (!st.empty()) {
            int u = st.back().first, i = st.back().second;
            if (i == (int)adj[u].size()) { st.pop_back(); continue; }
            st.back().second++;
            int v = adj[u][i];
            if (tin[v] < 0) { par[v] = u; dep[v] = dep[u] + 1; tin[v] = (int)order.size(); order.push_back(v); st.push_back({v, 0}); }
        }
        int n = (int)order.size();
        siz.assign(adj.size(), 1); kids.resize(adj.size());
        down.assign(dep.begin(), dep.end());
        for (int i = n - 1; i > 0; --i) siz[par[order[i]]] += siz[order[i]];
        for (int i = 1; i < n; ++i) kids[par[order[i]]].push_back(order[i]);
        sp.assign(1, vector<int>(n));
        for (int i = 0; i < n; ++i) sp[0][i] = par[order[i]] < 0 ? 0 : tin[par[order[i]]];
        for (int k = 1; (1 << k) <= n; ++k) {
            sp.emplace_back(n - (1 << k) + 1);
            for (int i = 0; i + (1 << k) <= n; ++i) sp[k][i] = min(sp[k - 1][i], sp[k - 1][i + (1 << (k - 1))]);
        }
    }
    // The same tree with a length on every edge: arcs[u] holds (v, length), in both directions or parent to child.
    static vector<vector<int>> plain(const vector<vector<pair<int, ll>>>& arcs) {
        vector<vector<int>> adj(arcs.size());
        for (size_t u = 0; u < arcs.size(); ++u) for (const auto& a : arcs[u]) adj[u].push_back(a.first);
        return adj;
    }
    RootedTree(const vector<vector<pair<int, ll>>>& arcs, int root) : RootedTree(plain(arcs), root) {
        for (int u : order) for (const auto& a : arcs[u]) if (par[a.first] == u) down[a.first] = down[u] + a.second;
    }
    ll length(int u, int v) const { return down[u] + down[v] - 2 * down[lca(u, v)]; }
    int lca(int u, int v) const {
        if (u == v) return u;
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
    template <class P> vector<P> chains(int a, int b) const {
        layout();
        vector<P> out;
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
// @part CpArrHash
struct CpArrHash {
    template <size_t K> size_t operator()(const array<ll, K>& a) const {
        uint64_t h = 0x9e3779b97f4a7c15ULL;
        for (ll x : a) {
            uint64_t z = (uint64_t)x + h + 0x9e3779b97f4a7c15ULL;
            z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
            z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
            h = z ^ (z >> 31);
        }
        return (size_t)h;
    }
};
// @end
// @part CpHash
struct CpHash {
    size_t operator()(ll x) const {
        uint64_t z = (uint64_t)x + 0x9e3779b97f4a7c15ULL;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        return (size_t)(z ^ (z >> 31));
    }
    // a record hashes its fields in order (each record struct has `cp_hash`)
    template <class T> auto operator()(const T& x) const -> decltype(x.cp_hash(*this)) { return x.cp_hash(*this); }
};
// @end
// @part CpCounter|CpDenseCounter
// Counter<K>: how many of each key, and how many keys have a count that is not zero. Stand-in for the
// library struct, whose `at` is a projection with a write-back (see tests/lend/writeback.cp).
template <class C, class K> struct CpCount {
    C* c; K k;
    operator ll() const { return c->get(k); }
    CpCount& operator=(ll v) { c->set(k, v); return *this; }
    CpCount& operator+=(ll d) { c->set(k, c->get(k) + d); return *this; }
    CpCount& operator-=(ll d) { c->set(k, c->get(k) - d); return *this; }
};
template <class M, class K> struct CpCounter {
    M m; ll distinct = 0;
    ll get(const K& k) const { auto it = m.find(k); return it == m.end() ? 0 : it->second; }
    void set(const K& k, ll v) {
        auto it = m.find(k);
        if (it == m.end()) { if (v != 0) { m.emplace(k, v); ++distinct; } }
        else if (v == 0) { m.erase(it); --distinct; }
        else it->second = v;
    }
    CpCount<CpCounter, K> at(const K& k) { return {this, k}; }
};
struct CpDenseCounter {
    vector<ll> m; ll distinct = 0;
    CpDenseCounter() {}
    explicit CpDenseCounter(ll n) : m(n, 0) {}
    ll get(int k) const { return m[k]; }
    void set(int k, ll v) { distinct += (v != 0) - (m[k] != 0); m[k] = v; }
    CpCount<CpDenseCounter, int> at(int k) { return {this, k}; }
};
// @end
// @part CpPrefix
// prefix(a): p[r][c] is the sum of the cells before boundary r and before boundary c (a 1-D array is one column)
struct CpPrefix {
    vector<ll> p; ll w = 0;
    CpPrefix() {}
    template <class A> CpPrefix(const A& a, ll rows, ll cols) : p((rows + 1) * (cols + 1), 0), w(cols + 1) {
        for (ll r = 0; r < rows; ++r) for (ll c = 0; c < cols; ++c)
            p[(r + 1) * w + c + 1] = p[r * w + c + 1] + p[(r + 1) * w + c] - p[r * w + c] + a[r * cols + c];
    }
    ll wrap(ll from, ll k, ll n, int line) const {     // 1-D: k cells starting at `from`, going round past the end
        if (k < 0 || n <= 0) cp_fail("sum(from:, len:) needs len >= 0 and a non-empty array", line);
        auto upto = [&](ll b) { return p[b * w + 1]; };
        ll rest = k % n, to = from + rest;
        return (k / n) * upto(n) + (to <= n ? upto(to) - upto(from) : upto(n) - upto(from) + upto(to - n));
    }
    ll sum(Rng rs, Rng cs) const {      // an empty range sums to 0
        if (rs.lo >= rs.hi || cs.lo >= cs.hi) return 0;
        return p[rs.hi * w + cs.hi] - p[rs.lo * w + cs.hi] - p[rs.hi * w + cs.lo] + p[rs.lo * w + cs.lo];
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
    map<ll, ll> d;                      // the change of value at each boundary
    void add(Rng r, ll x) { if (r.lo < r.hi && x != 0) { d[r.lo] += x; d[r.hi] -= x; } }
    template <class Tup> vector<Tup> runs(Rng clip) const {
        vector<Tup> out;
        if (clip.lo >= clip.hi) return out;
        ll cur = 0, at = clip.lo;
        auto it = d.begin();
        for (; it != d.end() && it->first <= clip.lo; ++it) cur += it->second;
        auto put = [&](ll hi) {
            if (hi <= at) return;
            if (!out.empty() && (ll)out.back().f1 == cur) out.back().f0.hi = hi; else out.push_back(Tup{Rng{at, hi}, cur});
            at = hi;
        };
        for (; it != d.end() && it->first < clip.hi; ++it) { put(it->first); cur += it->second; }
        put(clip.hi);
        return out;
    }
};
// @end
// @part cp_take
// take(place): the value, leaving the place empty
template <class T> static inline T cp_take(T& x, T empty) { T v = std::move(x); x = std::move(empty); return v; }
// @end
