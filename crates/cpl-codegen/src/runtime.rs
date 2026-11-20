//! The C++ the emitted program leans on: input, output, and the few
//! operations C++ spells differently. Each piece is emitted only when the
//! program uses it, so a small program is a small file.

/// One piece of the runtime: its name, the pieces it needs, and its code.
pub(crate) struct Piece {
    pub name: &'static str,
    pub needs: &'static [&'static str],
    pub code: &'static str,
}

/// Every piece, in the order they are emitted.
pub(crate) const PIECES: &[Piece] = &[
    Piece {
        name: "out",
        needs: &[],
        code: "\
// Output is collected here and written once at the end.
std::string out;
inline void flush() { fwrite(out.data(), 1, out.size(), stdout); out.clear(); }
inline void put_char(u8 c) { out.push_back((char)c); }
inline void put_str(const std::string& s) { out += s; }
inline void put_str(const char* s) { out += s; }
inline void put_line() { out.push_back('\\n'); if (out.size() >= (1u << 16)) flush(); }
",
    },
    Piece {
        name: "put_int",
        needs: &["out"],
        code: "\
inline void put_int(ll x) { char b[24]; out.append(b, (size_t)snprintf(b, sizeof b, \"%lld\", x)); }
",
    },
    Piece {
        name: "put_float",
        needs: &["out"],
        code: "\
inline void put_float(double x, int digits) { char b[400]; out.append(b, (size_t)snprintf(b, sizeof b, \"%.*f\", digits, x)); }
",
    },
    Piece {
        name: "capture",
        needs: &["out"],
        code: "\
// Collects what is printed while it lives: how a \"{x}\" string is built.
struct Capture {
    std::string saved;
    Capture() { saved.swap(out); }
    std::string take() { std::string text; text.swap(out); out.swap(saved); return text; }
};
",
    },
    Piece {
        name: "fail",
        needs: &["out"],
        code: "\
[[noreturn]] inline void fail(const char* why) { flush(); fputs(why, stderr); fputc('\\n', stderr); exit(3); }
",
    },
    Piece {
        name: "in",
        needs: &[],
        code: "\
// All of the input, read once.
std::string in;
size_t in_at = 0;
inline void read_all() { char b[1 << 16]; size_t n; while ((n = fread(b, 1, sizeof b, stdin)) > 0) in.append(b, n); }
inline bool is_space(char c) { return c == ' ' || c == '\\n' || c == '\\t' || c == '\\r' || c == '\\f'; }
inline void skip_space() { while (in_at < in.size() && is_space(in[in_at])) in_at++; }
",
    },
    Piece {
        name: "read_int",
        needs: &["in"],
        code: "\
inline ll read_int() {
    skip_space();
    bool negative = false;
    if (in_at < in.size() && (in[in_at] == '-' || in[in_at] == '+')) negative = in[in_at++] == '-';
    ll x = 0;  // built downward, so the smallest Int reads without overflow
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 - (in[in_at++] - '0');
    return negative ? x : -x;
}
",
    },
    Piece {
        name: "read_float",
        needs: &["in"],
        code: "\
inline double read_float() { skip_space(); char* end; double x = strtod(in.c_str() + in_at, &end); in_at = (size_t)(end - in.c_str()); return x; }
",
    },
    Piece {
        name: "read_char",
        needs: &["in"],
        code: "\
inline u8 read_char() { skip_space(); return in_at < in.size() ? (u8)in[in_at++] : 0; }
",
    },
    Piece {
        name: "read_str",
        needs: &["in"],
        code: "\
inline std::string read_str() { skip_space(); size_t start = in_at; while (in_at < in.size() && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
",
    },
    Piece {
        name: "ediv",
        needs: &[],
        code: "\
// Euclidean division: the remainder is never negative.
inline ll ediv(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs(\"division by zero\\n\", stderr); exit(3); }
    if (a == (-9223372036854775807LL - 1) && b == -1) { fputs(\"integer overflow\\n\", stderr); exit(3); }
#endif
    ll q = a / b, r = a % b; return q - ((r >> 63) & ((b >> 63) | 1));
}
",
    },
    Piece {
        name: "emod",
        needs: &[],
        code: "\
inline ll emod(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs(\"division by zero\\n\", stderr); exit(3); }
#endif
    if (b == -1) return 0;
    ll r = a % b, s = b >> 63; return r + ((r >> 63) & ((b ^ s) - s));
}
",
    },
    Piece {
        name: "divp",
        needs: &[],
        code: "\
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs(\"division by zero\\n\", stderr); exit(3); }
#endif
    return a / b + ((a % b) >> 63);
}
",
    },
    Piece {
        name: "modp",
        needs: &[],
        code: "\
inline ll modp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs(\"division by zero\\n\", stderr); exit(3); }
#endif
    ll r = a % b; return r + ((r >> 63) & b);
}
",
    },
    Piece {
        name: "ipow",
        needs: &[],
        code: "\
inline ll ipow(ll a, ll b) { ll r = 1; while (b > 0) { if (b & 1) r *= a; b >>= 1; if (b) a *= a; } return r; }
",
    },
    Piece {
        name: "grid",
        needs: &[],
        code: "\
// An array with K dimensions, stored flat in row-major order.
template <class T, int K> struct Grid {
    std::vector<T> d;
    ll n[K] = {};
    template <class... I> size_t at(I... i) const { ll ix[K] = {(ll)i...}; ll p = 0; for (int k = 0; k < K; k++) p = p * n[k] + ix[k]; return (size_t)p; }
    template <class... I> T& operator()(I... i) { return d[at(i...)]; }
    template <class... I> const T& operator()(I... i) const { return d[at(i...)]; }
    bool operator==(const Grid& o) const { return d == o.d; }
    bool operator<(const Grid& o) const { return d < o.d; }
};
template <class T, int K> Grid<T, K> grid(std::array<ll, K> n, const T& fill) {
    Grid<T, K> g; ll cells = 1;
    for (int k = 0; k < K; k++) { g.n[k] = n[k]; cells *= n[k]; }
    g.d.assign((size_t)cells, fill);
    return g;
}
",
    },
    Piece {
        name: "pop",
        needs: &[],
        code: "\
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
",
    },
    Piece {
        name: "first",
        needs: &[],
        code: "\
template <class T> std::optional<T> first(const std::vector<T>& v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
",
    },
    Piece {
        name: "last",
        needs: &[],
        code: "\
template <class T> std::optional<T> last(const std::vector<T>& v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
",
    },
    Piece {
        name: "has",
        needs: &[],
        code: "\
template <class T, class U> bool has(const std::vector<T>& v, const U& x) { return std::find(v.begin(), v.end(), x) != v.end(); }
",
    },
    Piece {
        name: "get",
        needs: &[],
        code: "\
template <class K, class V, class Q> std::optional<V> get(const std::map<K, V>& m, const Q& k) { auto it = m.find(k); return it == m.end() ? std::optional<V>() : std::optional<V>(it->second); }
",
    },
    Piece {
        name: "set_min",
        needs: &[],
        code: "\
template <class T> std::optional<T> set_min(const std::set<T>& s) { return s.empty() ? std::optional<T>() : std::optional<T>(*s.begin()); }
",
    },
    Piece {
        name: "set_max",
        needs: &[],
        code: "\
template <class T> std::optional<T> set_max(const std::set<T>& s) { return s.empty() ? std::optional<T>() : std::optional<T>(*s.rbegin()); }
",
    },
    Piece {
        name: "sorted",
        needs: &[],
        code: "\
template <class T> std::vector<T> sorted(std::vector<T> v) { std::sort(v.begin(), v.end()); return v; }
template <class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
",
    },
    Piece {
        name: "reversed",
        needs: &[],
        code: "\
template <class T> std::vector<T> reversed(std::vector<T> v) { std::reverse(v.begin(), v.end()); return v; }
",
    },
    Piece {
        name: "next_pos",
        needs: &[],
        code: "\
inline std::optional<int> next_pos(ll v, ll len) { return v + 1 < len ? std::optional<int>((int)(v + 1)) : std::optional<int>(); }
",
    },
    Piece {
        name: "prev_pos",
        needs: &[],
        code: "\
inline std::optional<int> prev_pos(ll v) { return v > 0 ? std::optional<int>((int)(v - 1)) : std::optional<int>(); }
",
    },
    Piece {
        name: "either",
        needs: &[],
        code: "\
template <class T> std::optional<T> either(std::optional<T> a, std::optional<T> b) { return a ? a : b; }
",
    },
    Piece {
        name: "cphash",
        needs: &[],
        code: "\
// Salted splitmix64 hash for HashMap / HashSet (defeats anti-hash tests).
struct CpHash {
    static uint64_t splitmix64(uint64_t x) {
        x += 0x9e3779b97f4a7c15ULL;
        x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
        x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
        return x ^ (x >> 31);
    }
    static uint64_t salt() {
        static const uint64_t s = (uint64_t)std::chrono::steady_clock::now().time_since_epoch().count();
        return s;
    }
    size_t operator()(ll x) const { return (size_t)splitmix64((uint64_t)x + salt()); }
    size_t operator()(int x) const { return (*this)((ll)x); }
    size_t operator()(u8 x) const { return (*this)((ll)x); }
    size_t operator()(bool x) const { return (*this)((ll)x); }
    size_t operator()(const std::string& s) const {
        uint64_t h = salt();
        for (unsigned char c : s) h = splitmix64(h ^ c);
        return (size_t)h;
    }
    template <class T> size_t operator()(const std::optional<T>& o) const {
        return o ? (*this)(*o) : (size_t)splitmix64(salt());
    }
    template <class T, size_t I = 0> uint64_t hash_tuple(const T& t) const {
        if constexpr (I == std::tuple_size<T>::value) return salt();
        else return splitmix64(hash_tuple<T, I + 1>(t) ^ (uint64_t)(*this)(std::get<I>(t)));
    }
    template <class... Ts> size_t operator()(const std::tuple<Ts...>& t) const {
        return (size_t)hash_tuple(t);
    }
    template <class T> auto operator()(const T& x) const -> decltype((*this)(x.cpl_tie())) {
        return (*this)(x.cpl_tie());
    }
};
",
    },
    Piece {
        name: "range",
        needs: &[],
        code: "\
template <class T> struct Range {
    T lo = 0, hi = 0;
    ll len() const { return hi > lo ? (ll)hi - (ll)lo : 0; }
    bool empty() const { return hi <= lo; }
    bool has(T x) const { return x >= lo && x < hi; }
    std::optional<T> first() const { return lo < hi ? std::optional<T>(lo) : std::nullopt; }
    std::optional<T> last() const { return lo < hi ? std::optional<T>((T)(hi - 1)) : std::nullopt; }
    std::vector<T> rev() const {
        std::vector<T> out;
        if (lo < hi) { out.reserve((size_t)((ll)hi - (ll)lo)); for (T x = hi; x-- > lo;) out.push_back(x); }
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
        name: "rng",
        needs: &[],
        code: "\
// Deterministic splitmix64 PRNG (matches the debug interpreter).
inline uint64_t rng_next() {
    static uint64_t s = 0x9e3779b97f4a7c15ULL;
    s += 0x9e3779b97f4a7c15ULL;
    uint64_t z = s;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}
inline ll rand_range(ll lo, ll hi) {
    unsigned __int128 span = (unsigned __int128)((__int128)hi - (__int128)lo + 1);
    return (ll)((__int128)lo + (__int128)(rng_next() % span));
}
template <class T> void shuffle(std::vector<T>& v) {
    for (size_t i = v.size(); i-- > 1;) std::swap(v[i], v[(size_t)(rng_next() % (i + 1))]);
}
",
    },
    Piece {
        name: "bits",
        needs: &[],
        code: "\
inline ll popcount(ll x) { return (ll)__builtin_popcountll((uint64_t)x); }
inline ll ctz(ll x) { return x == 0 ? 64 : (ll)__builtin_ctzll((uint64_t)x); }
inline ll clz(ll x) { return x == 0 ? 64 : (ll)__builtin_clzll((uint64_t)x); }
inline ll bit_width(ll x) { return x == 0 ? 0 : 64 - (ll)__builtin_clzll((uint64_t)x); }
inline ll bit_ceil(ll x) { return x <= 1 ? 1 : (1LL << (64 - __builtin_clzll((uint64_t)(x - 1)))); }
inline ll isqrt(ll x) { ll r = (ll)std::sqrt((double)x); while ((r + 1) <= 3037000499LL && (r + 1) * (r + 1) <= x) r++; while (r * r > x) r--; return r; }
",
    },
    Piece {
        name: "pop_val",
        needs: &["fail"],
        code: "\
template <class T> T pop_val(std::vector<T>& v) { if (v.empty()) fail(\"pop from an empty list\"); T x = std::move(v.back()); v.pop_back(); return x; }
",
    },
    Piece {
        name: "first_val",
        needs: &["fail"],
        code: "\
template <class T> T first_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
",
    },
    Piece {
        name: "last_val",
        needs: &["fail"],
        code: "\
template <class T> T last_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
",
    },
    Piece {
        name: "map_get_any",
        needs: &[],
        code: "\
template <class M, class Q> auto map_get(const M& m, const Q& k) -> std::optional<typename M::mapped_type> {
    auto it = m.find(k); return it == m.end() ? std::nullopt : std::optional<typename M::mapped_type>(it->second);
}
",
    },
    Piece {
        name: "map_fold",
        needs: &[],
        code: "\
template <class M, class K, class V> void map_max(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else if (v > it->second) it->second = v; }
template <class M, class K, class V> void map_min(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else if (v < it->second) it->second = v; }
template <class M, class K, class V> void map_mul(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else it->second *= v; }
template <class M, class K, class V> void map_and(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else it->second &= v; }
",
    },
    Piece {
        name: "pos_ops",
        needs: &["fail"],
        code: "\
inline std::optional<int> pos_try(ll v, ll lo, ll len) { return (v >= lo && v - lo < len) ? std::optional<int>((int)(v - lo)) : std::nullopt; }
inline std::optional<int> pos_try_ord(ll i, ll len) { return (i >= 0 && i < len) ? std::optional<int>((int)i) : std::nullopt; }
inline std::optional<int> pos_offset(ll i, ll d, ll len) { ll t = i + d; return (t >= 0 && t < len) ? std::optional<int>((int)t) : std::nullopt; }
inline int pos_cyc(ll i, ll d, ll len) { ll r = (i + d) % len; return (int)(r < 0 ? r + len : r); }
inline int bound_step(ll b, ll len) { if (b < 0 || b > len) fail(\"window leaves domain\"); return (int)b; }
inline std::optional<int> bound_left(ll b) { return b > 0 ? std::optional<int>((int)(b - 1)) : std::nullopt; }
inline std::optional<int> bound_right(ll b, ll len) { return b < len ? std::optional<int>((int)b) : std::nullopt; }
",
    },
    Piece {
        name: "str_conv",
        needs: &[],
        code: "\
inline std::vector<u8> str_to_list(const std::string& s) { return std::vector<u8>(s.begin(), s.end()); }
inline std::string list_to_str(const std::vector<u8>& v) { return std::string(v.begin(), v.end()); }
inline ll parse_int(const std::string& s) { return std::stoll(s); }
",
    },
];
