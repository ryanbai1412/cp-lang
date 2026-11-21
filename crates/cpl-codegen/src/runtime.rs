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
        name: "put_u64",
        needs: &["out"],
        code: "\
inline void put_u64(unsigned long long x) { char b[24]; out.append(b, (size_t)snprintf(b, sizeof b, \"%llu\", x)); }
",
    },
    Piece {
        name: "put_i128",
        needs: &["out"],
        code: "\
inline void put_i128(__int128 x) {
    char b[48]; int at = 48; bool negative = x < 0;
    unsigned __int128 u = negative ? -(unsigned __int128)x : (unsigned __int128)x;
    do { b[--at] = (char)('0' + (int)(u % 10)); u /= 10; } while (u);
    if (negative) b[--at] = '-';
    out.append(b + at, (size_t)(48 - at));
}
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
        name: "put_num",
        needs: &["out"],
        code: "\
// \"{n:05}\", \"{x:8.3f}\": the sign, then zeros or spaces up to the width.
inline void put_padded(const char* s, size_t n, bool zero, size_t width) {
    bool neg = n > 0 && s[0] == '-';
    if (zero) { if (neg) out.push_back('-'); if (n < width) out.append(width - n, '0'); out.append(s + neg, n - neg); }
    else { if (n < width) out.append(width - n, ' '); out.append(s, n); }
}
inline void put_num(ll x, int base, bool zero, int width) {
    char b[72]; int n = 0; unsigned long long u = x < 0 ? 0ULL - (unsigned long long)x : (unsigned long long)x;
    do { b[n++] = \"0123456789abcdef\"[u % base]; u /= base; } while (u);
    if (x < 0) b[n++] = '-';
    std::reverse(b, b + n); put_padded(b, (size_t)n, zero, (size_t)width);
}
inline void put_fixed(double x, int digits, bool zero, int width) {
    std::string b(32, 0); int n = snprintf(&b[0], b.size(), \"%.*f\", digits, x);
    if ((size_t)n >= b.size()) { b.assign((size_t)n + 1, 0); snprintf(&b[0], b.size(), \"%.*f\", digits, x); }
    // A value that rounds to zero has no sign: never -0.000.
    bool nonzero = false; for (int i = 0; i < n; i++) nonzero |= b[i] >= '1' && b[i] <= '9';
    bool drop = n > 0 && b[0] == '-' && !nonzero && x == x;
    put_padded(b.data() + drop, (size_t)(n - drop), zero, (size_t)width);
}
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
        name: "char_step",
        needs: &["fail"],
        code: "\
inline u8 char_step(ll c) { if (c < 0 || c > 127) fail(\"`Char + Int` left ASCII (0..=127)\"); return (u8)c; }
",
    },
    Piece {
        name: "narrow",
        needs: &["fail"],
        code: "\
// A value written as a smaller type must be one of that type's values.
inline u8 to_char(ll c) { if ((unsigned long long)c > 255) fail(\"not a byte, so not a `Char`\"); return (u8)c; }
inline ll to_int(__int128 x) { if (x < (__int128)(-9223372036854775807LL - 1) || x > (__int128)9223372036854775807LL) fail(\"an `I128` outside `Int`\"); return (ll)x; }
inline ll to_int(unsigned long long x) { if (x > 9223372036854775807ULL) fail(\"a `U64` outside `Int`\"); return (ll)x; }
inline ll to_int(double x) { if (!(x > -9223372036854775809.0 && x < 9223372036854775808.0)) fail(\"a float outside `Int`\"); return (ll)x; }
inline unsigned long long to_u64(__int128 x) { if (x < 0 || x > (__int128)~0ULL) fail(\"an `I128` outside `U64`\"); return (unsigned long long)x; }
",
    },
    Piece {
        name: "judge",
        needs: &["fail"],
        code: "\
// A value as the judge's grader holds it, and back: an Int is an `int`
// there and a Char a `char`. An Int too large for an `int` stops the program.
template <class T> struct Judge {
    using type = T;
    static T to(const T& v) { return v; }
    static T from(const T& v) { return v; }
};
template <> struct Judge<ll> {
    using type = int;
    static int to(ll v) { if (v != (int)v) fail(\"an Int given to the judge's grader does not fit in an int\"); return (int)v; }
    static ll from(int v) { return v; }
};
template <> struct Judge<u8> {
    using type = char;
    static char to(u8 v) { return (char)v; }
    static u8 from(char v) { return (u8)v; }
};
template <class T> struct Judge<std::vector<T>> {
    using type = std::vector<typename Judge<T>::type>;
    static type to(const std::vector<T>& v) { type r; r.reserve(v.size()); for (const T& x : v) r.push_back(Judge<T>::to(x)); return r; }
    static std::vector<T> from(const type& v) { std::vector<T> r; r.reserve(v.size()); for (const auto& x : v) r.push_back(Judge<T>::from(x)); return r; }
};
template <class... T> struct Judge<std::tuple<T...>> {
    using type = std::tuple<typename Judge<T>::type...>;
    static type to(const std::tuple<T...>& v) { return std::apply([](const T&... x) { return type(Judge<T>::to(x)...); }, v); }
    static std::tuple<T...> from(const type& v) { return std::apply([](const typename Judge<T>::type&... x) { return std::tuple<T...>(Judge<T>::from(x)...); }, v); }
};
",
    },
    Piece {
        name: "in",
        needs: &["fail"],
        code: "\
// The input so far. It comes in as the program asks for it, and what the
// program has printed goes out first: a judge that answers queries has
// to see the query before it writes the answer.
std::string in;
size_t in_at = 0;
inline bool is_space(char c) { return c == ' ' || c == '\\n' || c == '\\t' || c == '\\r' || c == '\\f'; }
inline bool more() { flush(); fflush(stdout); char b[1 << 16]; ssize_t n = read(0, b, sizeof b); if (n <= 0) return false; in.append(b, (size_t)n); return true; }
inline void skip_space() { for (;;) { while (in_at < in.size() && is_space(in[in_at])) in_at++; if (in_at < in.size() || !more()) return; } }
// Before a value is read: input that has run out stops the program, as it
// stops `cpl run`. The input comes in pieces, so the rest of the value is
// brought in too, up to the space after it.
inline void need_input() {
    skip_space();
    if (in_at >= in.size()) fail(\"the input ended where a value was expected\");
    if (in.size() - in_at >= 512) return;
    for (size_t i = in_at;; i++) { if (i >= in.size() && !more()) return; if (is_space(in[i])) return; }
}
// A number that has no digit where it should start is not one.
inline void need_digit() { if (in_at >= in.size() || in[in_at] < '0' || in[in_at] > '9') fail(\"the input has something else where a number was expected\"); }
",
    },
    Piece {
        name: "at_eof",
        needs: &["in"],
        code: "\
inline bool at_eof() { skip_space(); return in_at >= in.size(); }
",
    },
    Piece {
        name: "read_int",
        needs: &["in"],
        code: "\
inline ll read_int() {
    need_input();
    bool negative = false;
    if (in_at < in.size() && (in[in_at] == '-' || in[in_at] == '+')) negative = in[in_at++] == '-';
    need_digit();
    ll x = 0;  // built downward, so the smallest Int reads without overflow
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 - (in[in_at++] - '0');
    return negative ? x : -x;
}
",
    },
    Piece {
        name: "read_bool",
        needs: &["read_int", "fail"],
        code: "\
inline bool read_bool() { ll x = read_int(); if (x != 0 && x != 1) fail(\"a Bool is read as 0 or 1\"); return x == 1; }
",
    },
    Piece {
        name: "read_i128",
        needs: &["in"],
        code: "\
inline __int128 read_i128() {
    need_input();
    bool negative = false;
    if (in_at < in.size() && (in[in_at] == '-' || in[in_at] == '+')) negative = in[in_at++] == '-';
    need_digit();
    __int128 x = 0;
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 - (in[in_at++] - '0');
    return negative ? x : -x;
}
",
    },
    Piece {
        name: "read_u64",
        needs: &["in"],
        code: "\
inline unsigned long long read_u64() {
    need_input();
    if (in_at < in.size() && in[in_at] == '+') in_at++;
    need_digit();
    unsigned long long x = 0;
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 + (unsigned long long)(in[in_at++] - '0');
    return x;
}
",
    },
    Piece {
        name: "read_float",
        needs: &["in"],
        code: "\
inline double read_float() { need_input(); char* end; double x = strtod(in.c_str() + in_at, &end); if (end == in.c_str() + in_at) fail(\"the input has something else where a number was expected\"); in_at = (size_t)(end - in.c_str()); return x; }
",
    },
    Piece {
        name: "read_char",
        needs: &["in"],
        code: "\
inline u8 read_char() { need_input(); return in_at < in.size() ? (u8)in[in_at++] : 0; }
",
    },
    Piece {
        name: "read_str",
        needs: &["in"],
        code: "\
inline std::string read_str() { need_input(); size_t start = in_at; while ((in_at < in.size() || more()) && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
",
    },
    Piece {
        name: "ediv",
        needs: &[],
        code: "\
// Euclidean division: the remainder is never negative.
// A zero divisor and `Int.MIN / -1` stop every build: the judge's too, where
// they would be undefined and print whatever the machine left.
inline ll ediv(ll a, ll b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    if (__builtin_expect(a == (-9223372036854775807LL - 1) && b == -1, 0)) { cpl::flush(); fputs(\"integer overflow\\n\", stderr); exit(3); }
    ll q = a / b, r = a % b; return q - ((r >> 63) & ((b >> 63) | 1));
}
",
    },
    Piece {
        name: "emod",
        needs: &[],
        code: "\
inline ll emod(ll a, ll b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    if (b == -1) return 0;
    // `|b|` is 2^63 for `Int.MIN`, so it is added as an unsigned.
    typedef unsigned long long U;
    ll r = a % b; U s = (U)(b >> 63); return (ll)((U)r + ((U)(r >> 63) & (((U)b ^ s) - s)));
}
",
    },
    Piece {
        name: "divp",
        needs: &[],
        code: "\
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    return a / b + ((a % b) >> 63);
}
",
    },
    Piece {
        name: "modp",
        needs: &[],
        code: "\
inline ll modp(ll a, ll b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    ll r = a % b; return r + ((r >> 63) & b);
}
",
    },
    Piece {
        name: "ediv128",
        needs: &[],
        code: "\
inline __int128 ediv128(__int128 a, __int128 b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    if (__builtin_expect(b == -1 && a == (__int128)((unsigned __int128)1 << 127), 0)) { cpl::flush(); fputs(\"integer overflow\\n\", stderr); exit(3); }
    __int128 q = a / b, r = a % b; return r < 0 ? (b > 0 ? q - 1 : q + 1) : q;
}
",
    },
    Piece {
        name: "emod128",
        needs: &[],
        code: "\
inline __int128 emod128(__int128 a, __int128 b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    if (b == -1) return 0;
    __int128 r = a % b; return r < 0 ? (b > 0 ? r + b : r - b) : r;
}
",
    },
    Piece {
        name: "abs128",
        needs: &[],
        code: "\
inline __int128 abs128(__int128 x) { return x < 0 ? -x : x; }
",
    },
    Piece {
        name: "shl128",
        needs: &[],
        code: "\
inline __int128 shl128(__int128 a, ll b) {
    if (b < 0 || b >= 128) { cpl::flush(); fputs(\"a shift is outside 0..128\\n\", stderr); exit(3); }
#ifdef LOCAL
    if (a < 0) { cpl::flush(); fputs(\"`<<` of a negative number\\n\", stderr); exit(3); }
    if (b > 0 && (a >> (127 - b)) != 0) { cpl::flush(); fputs(\"integer overflow: the result does not fit in an `I128` (128 bits)\\n\", stderr); exit(3); }
#endif
    return (__int128)((unsigned __int128)a << b);
}
",
    },
    Piece {
        name: "shamt",
        needs: &[],
        code: "\
// How far a shift goes: inside 0..w, or the program stops.
inline ll shamt(ll b, ll w) {
    if (__builtin_expect((unsigned long long)b >= (unsigned long long)w, 0)) { cpl::flush(); fprintf(stderr, \"a shift by %lld is outside 0..%lld\\n\", b, w); exit(3); }
    return b;
}
",
    },
    Piece {
        name: "swap_at",
        needs: &[],
        code: "\
// `xs.swap(i, j)`: both inside the sequence, or the program stops.
template <class S> inline void swap_at(S& s, ll i, ll j) {
    unsigned long long n = s.size();
    if (__builtin_expect((unsigned long long)i >= n || (unsigned long long)j >= n, 0)) { cpl::flush(); fprintf(stderr, \"swap(%lld, %lld) is outside a list of length %llu\\n\", i, j, n); exit(3); }
    std::swap(s[i], s[j]);
}
",
    },
    Piece {
        name: "truncate",
        needs: &[],
        code: "\
// `xs.truncate(n)`: keep the first n, all of them when there are fewer. A negative n stops the program.
template <class S> inline void truncate(S& s, ll n) {
    if (__builtin_expect(n < 0, 0)) { cpl::flush(); fprintf(stderr, \"truncate to negative length %lld\\n\", n); exit(3); }
    if ((unsigned long long)n < s.size()) s.resize((size_t)n);
}
",
    },
    Piece {
        name: "have",
        needs: &["fail"],
        code: "\
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail(\"`!` on `none`: there is no value here\");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail(\"`!` on `none`: there is no value here\");
    return std::move(*o);
}
",
    },
    Piece {
        name: "udiv",
        needs: &[],
        code: "\
inline unsigned long long udiv(unsigned long long a, unsigned long long b) {
    if (b == 0) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    return a / b;
}
inline unsigned long long umod(unsigned long long a, unsigned long long b) {
    if (b == 0) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    return a % b;
}
",
    },
    Piece {
        name: "wpow",
        needs: &[],
        code: "\
template <class T> inline T wpow(T a, T b) {
    if (b < 0) { cpl::flush(); fputs(\"`**` needs a power that is not negative\\n\", stderr); exit(3); }
    T r = 1; while (b > 0) { if (b & 1) r *= a; b >>= 1; if (b) a *= a; } return r; }
",
    },
    Piece {
        name: "ipow",
        needs: &[],
        code: "\
inline ll ipow(ll a, ll b) {
    if (b < 0) { cpl::flush(); fputs(\"`**` needs a power that is not negative\\n\", stderr); exit(3); }
    ll r = 1; while (b > 0) { if (b & 1) r *= a; b >>= 1; if (b) a *= a; } return r; }
",
    },
    Piece {
        name: "grid",
        needs: &["fail"],
        code: "\
// An array with K dimensions, stored flat in row-major order.
template <class T, int K> struct Grid {
    std::vector<T> d;
    ll n[K] = {};
    template <class... I> size_t at(I... i) const { ll ix[K] = {(ll)i...}; ll p = 0; for (int k = 0; k < K; k++) p = p * n[k] + ix[k]; return (size_t)p; }
    template <class... I> T& operator()(I... i) { return d[at(i...)]; }
    template <class... I> const T& operator()(I... i) const { return d[at(i...)]; }
    // Indexed by an `Int` that nothing has proved in range.
    template <class... I> size_t in(I... i) const { ll ix[K] = {(ll)i...}; for (int k = 0; k < K; k++) if ((unsigned long long)ix[k] >= (unsigned long long)n[k]) fail(\"index outside the array\"); return at(i...); }
    template <class... I> T& ck(I... i) { return d[in(i...)]; }
    template <class... I> const T& ck(I... i) const { return d[in(i...)]; }
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
        name: "niche",
        needs: &[],
        code: "\
// `?V` held in one scalar: `none` is a value of S that no V is stored as, so
// there is no flag beside the value. Everything else is std::optional's: `none`
// sorts first, and nothing converts to or from it without being asked.
template <class T, class S, S NONE> struct Niche {
    S raw = NONE;
    Niche() = default;
    explicit Niche(T x) : raw((S)x) {}
    explicit Niche(const std::optional<T>& o) : raw(o ? (S)*o : NONE) {}
    // A `List<Bool>` keeps a `char` per flag, so what it hands back is a `std::optional<char>`.
    template <class U> explicit Niche(const std::optional<U>& o) : raw(o ? (S)(T)*o : NONE) {}
    bool has_value() const { return raw != NONE; }
    explicit operator bool() const { return raw != NONE; }
    T operator*() const { return (T)raw; }
    template <class U = S, class = std::enable_if_t<std::is_same<U, T>::value>> T& operator*() { return raw; }
    explicit operator std::optional<T>() const { return raw != NONE ? std::optional<T>((T)raw) : std::nullopt; }
    friend bool operator==(Niche a, Niche b) { return a.raw == b.raw; }
    friend bool operator!=(Niche a, Niche b) { return a.raw != b.raw; }
    friend bool operator<(Niche a, Niche b) { return b.has_value() && (!a.has_value() || *a < *b); }
    friend bool operator>(Niche a, Niche b) { return b < a; }
    friend bool operator<=(Niche a, Niche b) { return !(b < a); }
    friend bool operator>=(Niche a, Niche b) { return !(a < b); }
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
// `?(.., V, ..)`: a tuple that holds a position is `none` when that position
// is -1, so there is no flag beside the tuple.
template <class T, size_t K> struct OptTuple {
    T raw = empty();
    static T empty() { T t{}; std::get<K>(t) = -1; return t; }
    OptTuple() = default;
    explicit OptTuple(const T& x) : raw(x) {}
    explicit OptTuple(const std::optional<T>& o) : raw(o ? *o : empty()) {}
    bool has_value() const { return std::get<K>(raw) != -1; }
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
        name: "niche_int",
        needs: &["niche"],
        code: "\
// For tables whose values are proven never to be Int.MIN.
using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;
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
        name: "ordered_near",
        needs: &[],
        code: "\
// The nearest key of an ordered set, or entry of an ordered map, on one side of `k`.
template <class T> T near_entry(const std::set<T>&, const T& key) { return key; }
template <class K, class V> std::tuple<K, V> near_entry(const std::map<K, V>&, const std::pair<const K, V>& e) { return std::tuple<K, V>(e.first, e.second); }
template <class C> auto near_at(const C& c, typename C::const_iterator it) -> std::optional<decltype(near_entry(c, *it))> {
    if (it == c.end()) return std::nullopt;
    return near_entry(c, *it);
}
template <class C> auto near_before(const C& c, typename C::const_iterator it) -> std::optional<decltype(near_entry(c, *it))> {
    if (it == c.begin()) return std::nullopt;
    return near_entry(c, *std::prev(it));
}
template <class C, class Q> auto first_ge(const C& c, const Q& k) { return near_at(c, c.lower_bound(k)); }
template <class C, class Q> auto first_gt(const C& c, const Q& k) { return near_at(c, c.upper_bound(k)); }
template <class C, class Q> auto last_le(const C& c, const Q& k) { return near_before(c, c.upper_bound(k)); }
template <class C, class Q> auto last_lt(const C& c, const Q& k) { return near_before(c, c.lower_bound(k)); }
template <class C> auto first_of(const C& c) { return near_at(c, c.begin()); }
template <class C> auto last_of(const C& c) { return near_before(c, c.end()); }
template <class C> auto pop_first(C& c) { auto r = first_of(c); if (r) c.erase(c.begin()); return r; }
template <class C> auto pop_last(C& c) { auto r = last_of(c); if (r) c.erase(std::prev(c.end())); return r; }
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
        name: "either",
        needs: &[],
        code: "\
template <class O> O either(O a, O b) { return a ? a : b; }
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
    // Neighbouring keys share a salted block of 64, so a walk over x, x + 1, ... stays in cache.
    size_t operator()(ll x) const { return (size_t)((splitmix64(((uint64_t)x >> 6) + salt()) << 6) | ((uint64_t)x & 63)); }
    size_t operator()(int x) const { return (*this)((ll)x); }
    size_t operator()(u8 x) const { return (*this)((ll)x); }
    size_t operator()(bool x) const { return (*this)((ll)x); }
    size_t operator()(const std::string& s) const {
        uint64_t h = salt();
        for (unsigned char c : s) h = splitmix64(h ^ c);
        return (size_t)h;
    }
    // A list or a fixed array: its length, then each element in order.
    template <class L> auto operator()(const L& xs) const -> decltype(xs.begin(), xs.data(), size_t()) {
        uint64_t h = salt() ^ xs.size();
        for (const auto& x : xs) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
    // An optional, packed or not.
    template <class O> auto operator()(const O& o) const -> decltype(o.has_value(), size_t()) {
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
#ifdef CPL_GEN
// A `@gen` program built by `cpl stress`: each draw is a choice that is
// written down, and read back while the failing case is shrunk.
struct Choices {
    std::vector<uint64_t> list;
    size_t at = 0;
    bool replay = false;
    FILE* log = nullptr;
    uint64_t salt = 0;
    Choices() {
        if (const char* seed = std::getenv(\"CPL_SEED\")) {
            salt = std::strtoull(seed, nullptr, 10) * 0x9e3779b97f4a7c15ULL;
        }
        if (const char* path = std::getenv(\"CPL_CHOICES\")) {
            replay = true;
            unsigned long long x;
            if (FILE* f = std::fopen(path, \"r\")) {
                while (std::fscanf(f, \"%llu\", &x) == 1) list.push_back(x);
                std::fclose(f);
            }
        }
        if (const char* path = std::getenv(\"CPL_CHOICES_OUT\")) log = std::fopen(path, \"w\");
    }
    ~Choices() { if (log) std::fclose(log); }
};
inline uint64_t rng_below(unsigned __int128 span, int site) {
    static Choices c;
    uint64_t pick;
    if (c.replay) {
        pick = c.at < c.list.size() ? c.list[c.at++] : 0;
        if (pick >= span) pick = (uint64_t)(span - 1);
    } else {
        pick = (uint64_t)((rng_next() ^ c.salt) % span);
    }
    if (c.log) std::fprintf(c.log, \"%d %llu\\n\", site, (unsigned long long)pick);
    return pick;
}
// The line that draws tells the choices of one list from another's.
inline ll rand_range(ll lo, ll hi, int site = __builtin_LINE()) {
    unsigned __int128 span = (unsigned __int128)((__int128)hi - (__int128)lo + 1);
    return (ll)((__int128)lo + (__int128)rng_below(span, site));
}
template <class T> void shuffle(std::vector<T>& v, int site = __builtin_LINE()) {
    for (size_t i = v.size(); i-- > 1;) std::swap(v[i], v[(size_t)rng_below(i + 1, site)]);
}
#else
inline ll rand_range(ll lo, ll hi) {
    unsigned __int128 span = (unsigned __int128)((__int128)hi - (__int128)lo + 1);
    return (ll)((__int128)lo + (__int128)(rng_next() % span));
}
template <class T> void shuffle(std::vector<T>& v) {
    for (size_t i = v.size(); i-- > 1;) std::swap(v[i], v[(size_t)(rng_next() % (i + 1))]);
}
#endif
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
        name: "count_query",
        needs: &["fail"],
        code: "\
// `ask(.., limit: K)`: the judge allows `K` queries in each test case.
inline ll asked = 0;
inline void count_query(ll limit) { if (++asked > limit) fail((\"query limit exceeded: the limit is \" + std::to_string(limit)).c_str()); }
",
    },
    Piece {
        name: "row",
        needs: &["fail"],
        code: "\
// A row of exactly K cells held inside a struct. It is made from a list
// of K cells and is a list again to whatever takes it whole.
template <class T, int K> struct Row {
    std::array<T, K> a{};
    Row() = default;
    Row(const std::vector<T>& v) {
        if (v.size() != (size_t)K) fail(\"a row of another length than its field has\");
        std::copy(v.begin(), v.end(), a.begin());
    }
    std::vector<T> vec() const { return std::vector<T>(a.begin(), a.end()); }
    operator std::vector<T>() const { return vec(); }
    size_t size() const { return (size_t)K; }
    T& operator[](size_t i) { return a[i]; }
    const T& operator[](size_t i) const { return a[i]; }
    bool operator==(const Row& o) const { return a == o.a; }
    bool operator!=(const Row& o) const { return a != o.a; }
    bool operator<(const Row& o) const { return a < o.a; }
};
template <class T, int K> T& at(Row<T, K>& v, ll i) { if ((unsigned long long)i >= (unsigned long long)K) fail(\"index outside the list\"); return v.a[(size_t)i]; }
template <class T, int K> const T& at(const Row<T, K>& v, ll i) { if ((unsigned long long)i >= (unsigned long long)K) fail(\"index outside the list\"); return v.a[(size_t)i]; }
",
    },
    Piece {
        name: "steps",
        needs: &["fail"],
        code: "\
// How many values `lo, lo + by, ...` lie below `hi`. A step forward only.
inline ll steps(ll lo, ll hi, ll by) { if (by <= 0) fail(\"a range steps forward: `by` is not positive\"); return lo < hi ? (ll)(((unsigned long long)hi - (unsigned long long)lo - 1) / (unsigned long long)by) + 1 : 0; }
",
    },
    Piece {
        name: "at",
        needs: &["fail"],
        code: "\
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
inline char& at(std::string& s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
inline const char& at(const std::string& s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
// `V(x)` and `V.at(i)`: an ordinal that must be one of the domain's `len`.
inline int pos(ll ord, ll len) { if ((unsigned long long)ord >= (unsigned long long)len) fail(\"outside the domain\"); return (int)ord; }
",
    },
    Piece {
        name: "sort_by",
        needs: &[],
        code: "\
// `sort(by: key)`: each key is worked out once, in order, before any is
// compared; equal keys keep their order.
template <class S, class F> void sort_by(S& seq, F&& key, bool desc) {
    using K = std::decay_t<decltype(key(*seq.begin()))>;
    using T = typename S::value_type;
    std::vector<K> keys;
    keys.reserve(seq.size());
    for (const auto& x : seq) keys.push_back(key(x));
    std::vector<size_t> order(keys.size());
    for (size_t i = 0; i < order.size(); i++) order[i] = i;
    std::stable_sort(order.begin(), order.end(), [&](size_t a, size_t b) { return desc ? keys[b] < keys[a] : keys[a] < keys[b]; });
    std::vector<T> sorted;
    sorted.reserve(order.size());
    for (size_t i : order) sorted.push_back(std::move(seq[i]));
    std::move(sorted.begin(), sorted.end(), seq.begin());
}
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
