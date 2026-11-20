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
inline ll ediv(ll a, ll b) { ll q = a / b, r = a % b; return q - ((r >> 63) & ((b >> 63) | 1)); }
",
    },
    Piece {
        name: "emod",
        needs: &[],
        code: "\
inline ll emod(ll a, ll b) { ll r = a % b, s = b >> 63; return r + ((r >> 63) & ((b ^ s) - s)); }
",
    },
    Piece {
        name: "divp",
        needs: &[],
        code: "\
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) { return a / b + ((a % b) >> 63); }
",
    },
    Piece {
        name: "modp",
        needs: &[],
        code: "\
inline ll modp(ll a, ll b) { ll r = a % b; return r + ((r >> 63) & b); }
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
];
