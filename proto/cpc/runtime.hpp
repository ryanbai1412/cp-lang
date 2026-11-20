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

struct Rng { ll lo, hi; };

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

// SegTree<T>: point update, range fold with a user monoid (id, op). Prototype stand-in for std.segtree.
template <class T> struct SegTree {
    int n = 0; T id; std::function<T(const T&, const T&)> op; vector<T> t;
    SegTree() {}
    template <class F> SegTree(int n_, T id_, F f) : n(n_), id(id_), op(f), t(2 * max(n_, 1), id_) {}
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

// RootedTree: parent, preorder and O(1) lca of a tree given as adjacency lists. Prototype stand-in for std.tree.
struct RootedTree {
    vector<int> par, order, tin, dep; vector<vector<int>> sp;
    RootedTree() {}
    RootedTree(const vector<vector<int>>& adj, int root) : par(adj.size(), -1), tin(adj.size(), -1), dep(adj.size(), 0) {
        vector<int> st{root};
        while (!st.empty()) {
            int u = st.back(); st.pop_back();
            tin[u] = (int)order.size(); order.push_back(u);
            for (int i = (int)adj[u].size() - 1; i >= 0; --i) { int v = adj[u][i]; if (v != par[u]) { par[v] = u; dep[v] = dep[u] + 1; st.push_back(v); } }
        }
        int n = (int)order.size();
        sp.assign(1, vector<int>(n));
        for (int i = 0; i < n; ++i) sp[0][i] = par[order[i]] < 0 ? 0 : tin[par[order[i]]];
        for (int k = 1; (1 << k) <= n; ++k) {
            sp.emplace_back(n - (1 << k) + 1);
            for (int i = 0; i + (1 << k) <= n; ++i) sp[k][i] = min(sp[k - 1][i], sp[k - 1][i + (1 << (k - 1))]);
        }
    }
    int lca(int u, int v) const {
        if (u == v) return u;
        int l = min(tin[u], tin[v]) + 1, r = max(tin[u], tin[v]) + 1, k = 31 - __builtin_clz(r - l);
        return order[min(sp[k][l], sp[k][r - (1 << k)])];
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
