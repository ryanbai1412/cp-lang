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
#ifdef _WIN32
// Binary mode, so Windows writes \\n as \\n, not \\r\\n: the same bytes on every OS.
inline const int binary_stdout = _setmode(_fileno(stdout), _O_BINARY);
#endif
// Open `Capture`s: while one lives, `out` holds only what it collects, and
// writing it out early would put it before what was printed first.
inline int capturing = 0;
inline void flush() { fwrite(out.data(), 1, out.size(), stdout); out.clear(); }
inline void put_char(u8 c) { out.push_back((char)c); if (out.size() >= (1u << 16) && !capturing) flush(); }
inline void put_str(const std::string& s) { out += s; }
inline void put_str(const char* s) { out += s; }
inline void put_line() { out.push_back('\\n'); if (out.size() >= (1u << 16) && !capturing) flush(); }
",
    },
    Piece {
        name: "put_int",
        needs: &["out"],
        code: "\
inline void put_int(ll x) {
  unsigned long long u = x < 0 ? 0ULL - (unsigned long long)x : (unsigned long long)x;
  char b[24]; int i = 24;
  do { b[--i] = char('0' + u % 10); u /= 10; } while (u);
  if (x < 0) b[--i] = '-';
  out.append(b + i, (size_t)(24 - i));
}
",
    },
    Piece {
        name: "put_u64",
        needs: &["out"],
        code: "\
inline void put_u64(unsigned long long x) {
  char b[24]; int i = 24;
  do { b[--i] = char('0' + x % 10); x /= 10; } while (x);
  out.append(b + i, (size_t)(24 - i));
}
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
    Capture() { saved.swap(out); ++capturing; }
    std::string take() { std::string text; text.swap(out); out.swap(saved); --capturing; return text; }
};
",
    },
    Piece {
        name: "fail",
        needs: &["out"],
        code: "\
[[noreturn]] inline void fail(const char* why) { flush(); fputs(why, stderr); fputc('\\n', stderr); exit(3); }
// A key outside a view's keys never reads the parent around it (R07.4.5).
[[noreturn]] inline void key_fail(ll k, ll lo, ll n) { char why[96]; snprintf(why, sizeof why, \"key %lld is not one of the view's keys %lld..%lld\", k, lo, lo + n); fail(why); }
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
inline u8 to_char(ll c) { if ((unsigned long long)c > 127) { flush(); fprintf(stderr, \"%lld is outside `0..=127`\\n\", c); exit(3); } return (u8)c; }
inline ll to_int(__int128 x) { if (x < -(__int128)9223372036854775807LL || x > (__int128)9223372036854775807LL) fail(\"an `I128` outside `Int`\"); return (ll)x; }
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
inline bool is_space(char c) { return (unsigned char)c <= ' ' && (c == ' ' || (unsigned char)(c - '\\t') <= 4); }
inline bool more() { if (!out.empty()) { flush(); fflush(stdout); } if (in_at >= in.size()) { in.clear(); in_at = 0; } char b[1 << 16]; ssize_t n = read(0, b, sizeof b); if (n <= 0) return false; in.append(b, (size_t)n); return true; }
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
// A number starts with a digit and ends at a space or the end, so `12x` is
// not one. `in[in.size()]` is '\\0', so a digit scan stops there unchecked.
inline bool digit(char c) { return (unsigned char)(c - '0') <= 9; }
inline void need_digit(bool end = false) { if (end ? in_at < in.size() && !is_space(in[in_at]) : !digit(in[in_at])) fail(\"the input has something else where a number was expected\"); }
",
    },
    // `s.parse<T>()` reads from `s` as if it were the input. With it in the
    // program, `fail` throws while a string is being read and `more` stops
    // at its end (see `Emitter::piece_code`).
    Piece {
        name: "parse_str",
        needs: &["in"],
        code: "\
struct parse_scope {
    std::string held; size_t held_at;
    explicit parse_scope(const std::string& s) : held(std::move(in)), held_at(in_at) { in = s; in_at = 0; parsing++; }
    ~parse_scope() { in = std::move(held); in_at = held_at; parsing--; }
    bool whole() { skip_space(); return in_at >= in.size(); }
};
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
        needs: &["in", "fail"],
        code: "\
inline ll read_int() {
    need_input();
    bool negative = false;
    if (in[in_at] == '-' || in[in_at] == '+') negative = in[in_at++] == '-';
    need_digit();
    ll x = 0;  // built downward, so -2^63 reads without overflow
    while (digit(in[in_at])) x = x * 10 - (in[in_at++] - '0');
    need_digit(true);
    if (x == (-0x7fffffffffffffffLL - 1)) fail(\"a number in the input does not fit in an `Int`\");
    return negative ? x : -x;
}
",
    },
    Piece {
        name: "read_in",
        needs: &["read_int", "fail"],
        code: "\
inline ll read_in(ll lo, ll hi) { ll x = read_int(); if (x < lo || x > hi) fail(\"a number in the input is outside its range\"); return x; }
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
    if (in[in_at] == '-' || in[in_at] == '+') negative = in[in_at++] == '-';
    need_digit();
    __int128 x = 0;
    while (digit(in[in_at])) x = x * 10 - (in[in_at++] - '0');
    need_digit(true);
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
    if (in[in_at] == '+') in_at++;
    need_digit();
    unsigned long long x = 0;
    while (digit(in[in_at])) x = x * 10 + (unsigned long long)(in[in_at++] - '0');
    need_digit(true);
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
inline u8 read_char() {
  if (in_at < in.size() && !is_space(in[in_at])) return (u8)in[in_at++];
  need_input(); return in_at < in.size() ? (u8)in[in_at++] : 0;
}
",
    },
    Piece {
        name: "read_chars",
        needs: &["in"],
        code: "\
// Fills `v` with the next `v.size()` chars that are not spaces, like that
// many `read_char()`s in one pass.
template <class V> void read_chars(V& v) {
  for (size_t k = 0, m = v.size(); k < m;) {
    if (in_at >= in.size()) { if (!more()) fail(\"the input ended where a value was expected\"); continue; }
    // Copies the run of chars before the next space with no refill test per char.
    const char* p = in.data() + in_at;
    size_t e = std::min(in.size() - in_at, m - k), j = 0;
    for (; j < e && !is_space(p[j]); j++) v[k + j] = (typename V::value_type)(u8)p[j];
    k += j, in_at += j + (j < e);
  }
}
",
    },
    Piece {
        name: "read_str",
        needs: &["in"],
        code: "\
inline void read_str_into(std::string& r) {
    need_input();
    r.clear();
    for (;;) {
        const char* p = in.data(); size_t n = in.size(), st = in_at;
        while (in_at < n && !is_space(p[in_at])) in_at++;
        r.append(p + st, in_at - st);
        if (in_at < n || !more()) return;
    }
}
inline std::string read_str() { std::string r; read_str_into(r); return r; }
",
    },
    Piece {
        name: "read_str_in",
        needs: &["read_str", "fail"],
        code: "\
inline std::string read_str_in(ll lo, ll hi, const char* alphabet) {
    std::string r = read_str();
    if ((ll)r.size() < lo || (ll)r.size() > hi) fail(\"a string in the input has a length outside its range\");
    for (char c : r) {
        const char* a = alphabet;
        while (a && *a && *a != c) a++;
        if (a && !*a) fail(\"a string in the input has a character outside its alphabet\");
    }
    return r;
}
",
    },
    Piece {
        name: "read_line",
        needs: &["in"],
        code: "\
inline std::string read_line() {
    bool after_tok = in_at > 0 && in[in_at - 1] != '\\n';
    if (after_tok) {
        if (in_at >= in.size()) more();
        if (in_at < in.size() && in[in_at] == '\\r') {
            if (in_at + 1 >= in.size()) more();
            if (in_at + 1 < in.size() && in[in_at + 1] == '\\n') in_at += 2;
        } else if (in_at < in.size() && in[in_at] == '\\n') {
            in_at++;
        }
    }
    std::string r;
    for (;;) {
        if (in_at >= in.size() && !more()) break;
        const char* p = in.data(); size_t n = in.size(), st = in_at;
        while (in_at < n && p[in_at] != '\\n') in_at++;
        r.append(p + st, in_at - st);
        if (in_at < n) { in_at++; break; }
    }
    if (!r.empty() && r.back() == '\\r') r.pop_back();
    return r;
}
",
    },
    Piece {
        name: "ediv",
        needs: &[],
        code: "\
// Euclidean division: the remainder is never negative.
// A zero divisor stops every build: the judge's too, where it would be
// undefined and print whatever the machine left.
inline ll ediv(ll a, ll b) {
    if (__builtin_expect(b == 0, 0)) { cpl::flush(); fputs(\"division by zero\\n\", stderr); exit(3); }
    if (__builtin_expect(((uint64_t)a | (uint64_t)b) <= 0xFFFFFFFFULL, 1)) return (ll)((uint32_t)a / (uint32_t)b);
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
    ll r = a % b; return r + ((r >> 63) & (b < 0 ? -b : b));
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
        name: "fsqrt",
        needs: &["out"],
        code: "\
inline double fsqrt(double x) {
#ifdef LOCAL
    if (x < 0.0) { cpl::flush(); fprintf(stderr, \"sqrt(%g) of a negative number\\n\", x); exit(3); }
#endif
    return std::sqrt(x);
}
",
    },
    Piece {
        name: "fln",
        needs: &["out"],
        code: "\
inline double fln(double x) {
#ifdef LOCAL
    if (x < 0.0) { cpl::flush(); fprintf(stderr, \"ln(%g) of a negative number\\n\", x); exit(3); }
#endif
    return std::log(x);
}
",
    },
    Piece {
        name: "farc",
        needs: &["out"],
        code: "\
inline void farc_check(const char* f, double x) {
#ifdef LOCAL
    if (!(x >= -1.0 && x <= 1.0)) { cpl::flush(); fprintf(stderr, \"%s(%g) is outside -1..=1\\n\", f, x); exit(3); }
#endif
}
inline double fasin(double x) { farc_check(\"asin\", x); return std::asin(x); }
inline double facos(double x) { farc_check(\"acos\", x); return std::acos(x); }
",
    },
    Piece {
        name: "check_range",
        needs: &["out"],
        code: "\
inline ll check_range(ll v, ll lo, ll hi, const char* text) {
#ifdef LOCAL
    if (__builtin_expect(v < lo || v > hi, 0)) { cpl::flush(); fprintf(stderr, \"%lld is outside `%s`\\n\", v, text); exit(3); }
#else
    (void)lo; (void)hi; (void)text;
#endif
    return v;
}
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
        name: "list_of",
        needs: &[],
        code: "\
// `[a, b]` of lists: the elements are built in order, as in `{a, b}`, then
// moved, not copied, into the list.
template <class T, size_t N> std::vector<T> list_of(T (&&a)[N]) {
    return std::vector<T>(std::make_move_iterator(a), std::make_move_iterator(a + N));
}
",
    },
    Piece {
        name: "reserve",
        needs: &[],
        code: "\
// `xs.reserve(n)`: reserve capacity for at least n elements. A negative n stops the program.
template <class S> inline void reserve(S& s, ll n) {
    if (__builtin_expect(n < 0, 0)) { cpl::flush(); fprintf(stderr, \"reserve to negative capacity %lld\\n\", n); exit(3); }
    s.reserve((size_t)n);
}
",
    },
    Piece {
        name: "have",
        needs: &["fail"],
        code: "\
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail(\"`!` on `None`: there is no value here\");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail(\"`!` on `None`: there is no value here\");
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
    template <class... I> size_t in(I... i) const { ll ix[K] = {(ll)i...}, p = 0; bool bad = 0; for (int k = 0; k < K; k++) bad |= (unsigned long long)ix[k] >= (unsigned long long)n[k], p = p * n[k] + ix[k]; if (bad) fail(\"index outside the array\"); return p; }
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
        name: "pop",
        needs: &[],
        code: "\
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
inline std::optional<u8> pop(std::string& s) { if (s.empty()) return std::nullopt; u8 x = (u8)s.back(); s.pop_back(); return x; }
",
    },
    Piece {
        name: "iarith",
        needs: &["fail"],
        code: "\
#ifdef LOCAL
template <class T> [[noreturn]] void overflow(T a, char op, T b) {
    char why[128];
    if constexpr (sizeof(T) == 8) snprintf(why, sizeof why, \"integer overflow: `%lld %c %lld` does not fit in an `Int` (64 bits)\", (ll)a, op, (ll)b);
    else snprintf(why, sizeof why, \"integer overflow: the result of `%c` does not fit in an `I128` (128 bits)\", op);
    fail(why);
}
inline ll iadd(ll a, ll b) { ll r; if (__builtin_add_overflow(a, b, &r) || r == std::numeric_limits<ll>::min()) overflow(a, '+', b); return r; }
inline ll isub(ll a, ll b) { ll r; if (__builtin_sub_overflow(a, b, &r) || r == std::numeric_limits<ll>::min()) overflow(a, '-', b); return r; }
inline ll imul(ll a, ll b) { ll r; if (__builtin_mul_overflow(a, b, &r) || r == std::numeric_limits<ll>::min()) overflow(a, '*', b); return r; }
inline __int128 iadd128(__int128 a, __int128 b) { __int128 r; if (__builtin_add_overflow(a, b, &r)) overflow(a, '+', b); return r; }
inline __int128 isub128(__int128 a, __int128 b) { __int128 r; if (__builtin_sub_overflow(a, b, &r)) overflow(a, '-', b); return r; }
inline __int128 imul128(__int128 a, __int128 b) { __int128 r; if (__builtin_mul_overflow(a, b, &r)) overflow(a, '*', b); return r; }
#else
inline ll iadd(ll a, ll b) { return (ll)((unsigned long long)a + (unsigned long long)b); }
inline ll isub(ll a, ll b) { return (ll)((unsigned long long)a - (unsigned long long)b); }
inline ll imul(ll a, ll b) { return (ll)((unsigned long long)a * (unsigned long long)b); }
typedef unsigned __int128 u128;
inline __int128 iadd128(__int128 a, __int128 b) { return (__int128)((u128)a + (u128)b); }
inline __int128 isub128(__int128 a, __int128 b) { return (__int128)((u128)a - (u128)b); }
inline __int128 imul128(__int128 a, __int128 b) { return (__int128)((u128)a * (u128)b); }
#endif
",
    },
    Piece {
        name: "first",
        needs: &[],
        code: "\
template <class C> auto first(const C& v) { using T = std::decay_t<decltype(*std::begin(v))>; return v.empty() ? std::optional<T>() : std::optional<T>(*std::begin(v)); }
inline std::optional<u8> first(const std::string& s) { return s.empty() ? std::optional<u8>() : std::optional<u8>((u8)s.front()); }
",
    },
    Piece {
        name: "last",
        needs: &[],
        code: "\
template <class C> auto last(const C& v) { using T = std::decay_t<decltype(*std::begin(v))>; return v.empty() ? std::optional<T>() : std::optional<T>(*std::prev(std::end(v))); }
inline std::optional<u8> last(const std::string& s) { return s.empty() ? std::optional<u8>() : std::optional<u8>((u8)s.back()); }
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
    // Full salted splitmix64 so both prime-mod and power-of-two open-addressing tables stay hack-resistant.
    size_t operator()(ll x) const { return (size_t)splitmix64((uint64_t)x + salt()); }
    size_t operator()(int x) const { return (*this)((ll)x); }
    size_t operator()(u8 x) const { return (*this)((ll)x); }
    size_t operator()(bool x) const { return (*this)((ll)x); }
    size_t operator()(const std::string& s) const {
        uint64_t h = salt();
        for (unsigned char c : s) h = splitmix64(h ^ c);
        return (size_t)h;
    }
    // A string view: same hash as `std::string`.
    template <class S> auto operator()(const S& s) const -> decltype(s.cpl_str_view_tag(), size_t()) {
        uint64_t h = salt();
        for (u8 c : s) h = splitmix64(h ^ (unsigned char)c);
        return (size_t)h;
    }
    // A list, list view, or fixed array: its length, then each element in order.
    template <class L> auto operator()(const L& xs) const -> decltype(xs.begin(), xs.size(), xs.cpl_list_view_tag(), size_t()) {
        uint64_t h = salt() ^ xs.size();
        for (const auto& x : xs) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
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
        name: "cphashmap",
        needs: &["cphash", "fail"],
        code: "\
// Open-addressing hash map with 7-bit fingerprints and backward-shift deletion.
template <class K, class V, class H = CpHash> struct HashMap {
    using key_type = K; using mapped_type = V; using value_type = std::pair<K, V>;
    std::vector<uint8_t> st; std::vector<value_type> kv; size_t n = 0;
    template <bool C> struct It {
        using P = std::conditional_t<C, const value_type, value_type>;
        const uint8_t* s; P* p; size_t i, cap;
        void skip() { while (i < cap && !s[i]) ++i; }
        P& operator*() const { return p[i]; }
        P* operator->() const { return p + i; }
        It& operator++() { ++i; skip(); return *this; }
        bool operator==(const It& o) const { return i == o.i; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    using Iter = It<false>; using CIter = It<true>;
    size_t size() const { return n; }
    bool empty() const { return n == 0; }
    void clear() {
        if (!n) return;
        if (st.size() > 64 && n * 8 < st.size()) { st.clear(); kv.clear(); }
        else std::fill(st.begin(), st.end(), 0);
        n = 0;
    }
    void rehash(size_t cap) {
        std::vector<uint8_t> ns(cap, 0); std::vector<value_type> nk(cap); size_t m = cap - 1;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = H{}(kv[i].first) & m;
            while (ns[j]) j = (j + 1) & m;
            ns[j] = st[i]; nk[j] = std::move(kv[i]);
        }
        st.swap(ns); kv.swap(nk);
    }
    size_t locate(const K& k) const {
        if (st.empty()) return 0;
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && kv[i].first == k) return i; i = (i + 1) & m; }
        return st.size();
    }
    std::pair<size_t, bool> slot(const K& k) {
        if (2 * (n + 1) > st.size()) { K kc = k; rehash(st.empty() ? 8 : st.size() * 2); return slot(kc); }
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && kv[i].first == k) return {i, false}; i = (i + 1) & m; }
        st[i] = fp; kv[i].first = k; ++n; return {i, true};
    }
    Iter begin() { Iter it{st.data(), kv.data(), 0, st.size()}; it.skip(); return it; }
    Iter end() { return {st.data(), kv.data(), st.size(), st.size()}; }
    CIter begin() const { CIter it{st.data(), kv.data(), 0, st.size()}; it.skip(); return it; }
    CIter end() const { return {st.data(), kv.data(), st.size(), st.size()}; }
    Iter find(const K& k) { return {st.data(), kv.data(), locate(k), st.size()}; }
    CIter find(const K& k) const { return {st.data(), kv.data(), locate(k), st.size()}; }
    size_t count(const K& k) const { return locate(k) != st.size() ? 1 : 0; }
    V& at(const K& k) { size_t i = locate(k); if (i == st.size()) fail(\"key not in map\"); return kv[i].second; }
    const V& at(const K& k) const { size_t i = locate(k); if (i == st.size()) fail(\"key not in map\"); return kv[i].second; }
    V& operator[](const K& k) { auto [i, nw] = slot(k); if (nw) kv[i].second = V{}; return kv[i].second; }
    std::pair<Iter, bool> emplace(const K& k, const V& v) {
        if (2 * (n + 1) > st.size()) { K kc = k; V vc = v; rehash(st.empty() ? 8 : st.size() * 2); return emplace(kc, vc); }
        auto [i, nw] = slot(k); if (nw) kv[i].second = v;
        return {{st.data(), kv.data(), i, st.size()}, nw};
    }
    void erase_at(size_t i) {
        --n; size_t m = st.size() - 1, j = i;
        for (;;) {
            j = (j + 1) & m; if (!st[j]) break;
            size_t home = H{}(kv[j].first) & m;
            if (((j - home) & m) >= ((j - i) & m)) { st[i] = st[j]; kv[i] = std::move(kv[j]); i = j; }
        }
        st[i] = 0; kv[i] = {};
    }
    void erase(Iter it) { erase_at(it.i); }
    size_t erase(const K& k) {
        size_t i = locate(k); if (i == st.size()) return 0;
        erase_at(i); return 1;
    }
    bool operator==(const HashMap& o) const {
        if (n != o.n) return false;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = o.locate(kv[i].first);
            if (j == o.st.size() || !(o.kv[j].second == kv[i].second)) return false;
        }
        return true;
    }
    bool operator!=(const HashMap& o) const { return !(*this == o); }
};
",
    },
    Piece {
        name: "cphashset",
        needs: &["cphash"],
        code: "\
// Open-addressing hash set with 7-bit fingerprints and backward-shift deletion.
template <class K, class H = CpHash> struct HashSet {
    using key_type = K; using value_type = K;
    std::vector<uint8_t> st; std::vector<std::array<K, 1>> ks; size_t n = 0;
    struct CIter {
        const uint8_t* s; const std::array<K, 1>* p; size_t i, cap;
        void skip() { while (i < cap && !s[i]) ++i; }
        const K& operator*() const { return p[i][0]; }
        const K* operator->() const { return &p[i][0]; }
        CIter& operator++() { ++i; skip(); return *this; }
        bool operator==(const CIter& o) const { return i == o.i; }
        bool operator!=(const CIter& o) const { return i != o.i; }
    };
    size_t size() const { return n; }
    bool empty() const { return n == 0; }
    void clear() {
        if (!n) return;
        if (st.size() > 64 && n * 8 < st.size()) { st.clear(); ks.clear(); }
        else std::fill(st.begin(), st.end(), 0);
        n = 0;
    }
    void rehash(size_t cap) {
        std::vector<uint8_t> ns(cap, 0); std::vector<std::array<K, 1>> nk(cap); size_t m = cap - 1;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = H{}(ks[i][0]) & m;
            while (ns[j]) j = (j + 1) & m;
            ns[j] = st[i]; nk[j][0] = std::move(ks[i][0]);
        }
        st.swap(ns); ks.swap(nk);
    }
    size_t locate(const K& k) const {
        if (st.empty()) return 0;
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && ks[i][0] == k) return i; i = (i + 1) & m; }
        return st.size();
    }
    CIter begin() const { CIter it{st.data(), ks.data(), 0, st.size()}; it.skip(); return it; }
    CIter end() const { return {st.data(), ks.data(), st.size(), st.size()}; }
    CIter find(const K& k) const { return {st.data(), ks.data(), locate(k), st.size()}; }
    size_t count(const K& k) const { return locate(k) != st.size() ? 1 : 0; }
    std::pair<CIter, bool> insert(const K& k) {
        if (2 * (n + 1) > st.size()) { K kc = k; rehash(st.empty() ? 8 : st.size() * 2); return insert(kc); }
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && ks[i][0] == k) return {{st.data(), ks.data(), i, st.size()}, false}; i = (i + 1) & m; }
        st[i] = fp; ks[i][0] = k; ++n; return {{st.data(), ks.data(), i, st.size()}, true};
    }
    size_t erase(const K& k) {
        size_t i = locate(k); if (i == st.size()) return 0;
        --n; size_t m = st.size() - 1, j = i;
        for (;;) {
            j = (j + 1) & m; if (!st[j]) break;
            size_t home = H{}(ks[j][0]) & m;
            if (((j - home) & m) >= ((j - i) & m)) { st[i] = st[j]; ks[i][0] = std::move(ks[j][0]); i = j; }
        }
        st[i] = 0; ks[i][0] = K{}; return 1;
    }
    bool operator==(const HashSet& o) const {
        if (n != o.n) return false;
        for (size_t i = 0; i < st.size(); ++i) if (st[i] && !o.count(ks[i][0])) return false;
        return true;
    }
    bool operator!=(const HashSet& o) const { return !(*this == o); }
};
",
    },
    Piece {
        name: "range",
        needs: &["fail"],
        code: "\
template <class T> struct Range {
    T lo = 0, hi = 0;
    static Range make(T lo, T hi) {
        using U = std::conditional_t<sizeof(T) == 16, unsigned __int128, unsigned long long>;
        if (hi > lo && (U)hi - (U)lo > (~(U)0 >> 1)) fail(\"range span overflow\");
        return {lo, hi};
    }
    ll len() const {
        if (hi <= lo) return 0;
        if constexpr (sizeof(T) == 16) {
            auto d = (unsigned __int128)hi - (unsigned __int128)lo;
            if (d > (~0ULL >> 1)) fail(\"range length overflow\");
            return (ll)d;
        }
        return (ll)hi - (ll)lo;
    }
    bool empty() const { return hi <= lo; }
    bool has(T x) const { return x >= lo && x < hi; }
    std::optional<T> first() const { return lo < hi ? std::optional<T>(lo) : std::nullopt; }
    std::optional<T> last() const { return lo < hi ? std::optional<T>((T)(hi - 1)) : std::nullopt; }
    T operator[](ll i) const { return (T)((ll)lo + i); }
    std::optional<T> get(ll i) const { return i < 0 || i >= len() ? std::optional<T>() : std::optional<T>((T)((ll)lo + i)); }
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
        uint64_t r = rng_next() ^ c.salt;
        pick = span >> 64 ? r : r % (uint64_t)span;
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
    uint64_t r = rng_next();
    return (ll)((uint64_t)lo + (span >> 64 ? r : r % (uint64_t)span));
}
template <class T> void shuffle(std::vector<T>& v) {
    for (size_t i = v.size(); i-- > 1;) std::swap(v[i], v[(size_t)(rng_next() % (i + 1))]);
}
#endif
",
    },
    Piece {
        name: "bits",
        needs: &["fail"],
        code: "\
inline ll popcount(ll x) { return (ll)__builtin_popcountll((uint64_t)x); }
inline ll ctz(ll x) { return x == 0 ? 64 : (ll)__builtin_ctzll((uint64_t)x); }
inline ll clz(ll x) { return x == 0 ? 64 : (ll)__builtin_clzll((uint64_t)x); }
inline ll bit_width(ll x) { return x == 0 ? 0 : 64 - (ll)__builtin_clzll((uint64_t)x); }
inline ll bit_ceil(ll x) { if (x > (1LL << 62)) fail(\"integer overflow: `bit_ceil` does not fit in an `Int` (64 bits)\"); return x <= 1 ? 1 : (1LL << (64 - __builtin_clzll((uint64_t)(x - 1)))); }
",
    },
    Piece {
        name: "no_fn",
        needs: &["fail"],
        code: "\
// The fn field of a struct the program never builds: no value of the
// struct exists, so it is never called.
template <class R> struct NoFn {
    template <class... A> [[noreturn]] R operator()(A&&...) const { fail(\"a fn field of a struct that is never built was called\"); }
};
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
    static Row filled(const T& x) { Row r; r.a.fill(x); return r; }
    Row(const std::vector<T>& v) {
        if (v.size() != (size_t)K) fail(\"a row of another length than its field has\");
        std::copy(v.begin(), v.end(), a.begin());
    }
    std::vector<T> vec() const { return std::vector<T>(a.begin(), a.end()); }
    operator std::vector<T>() const { return vec(); }
    size_t size() const { return (size_t)K; }
    auto begin() const { return a.begin(); }
    auto end() const { return a.end(); }
    auto rbegin() const { return a.rbegin(); }
    auto rend() const { return a.rend(); }
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
        name: "at_array",
        needs: &["fail"],
        code: "\
template <class T, size_t K> T& at(std::array<T, K>& v, ll i) { if ((unsigned long long)i >= K) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T, size_t K> const T& at(const std::array<T, K>& v, ll i) { if ((unsigned long long)i >= K) fail(\"index outside the list\"); return v[(size_t)i]; }
",
    },
    Piece {
        name: "filled_array",
        needs: &[],
        code: "\
template <class T, size_t K> inline std::array<T, K> filled_array(const T& x) { std::array<T, K> a; a.fill(x); return a; }
",
    },
    Piece {
        name: "sort_by",
        needs: &[],
        code: "\
// `sort(by: key)`: each key is worked out once, in order, before any is
// compared; equal keys keep their order.
// A `()` key (a `by:` left at `_ => ()`) makes every key equal.
template <class S, class F> void sort_by(S& seq, F&& key, bool desc) {
    using K = std::decay_t<decltype(key(*seq.begin()))>;
    using T = typename S::value_type;
    if constexpr (std::is_void_v<K>) {
        for (const auto& x : seq) key(x);
        (void)desc;
        return;
    } else {
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
}
",
    },
    Piece {
        name: "sort_cmp",
        needs: &["fail"],
        code: "\
// `sort(cmp: lt)`: stable O(n log n) comparator sort. Under `LOCAL` (`--checked`),
// traps on non-strict-weak-order comparators (irreflexivity, asymmetry, intransitivity).
template <class S, class F> void sort_cmp(S& seq, F&& cmp, bool desc) {
    auto lt = [&](const auto& a, const auto& b) -> bool {
        bool r = desc ? cmp(b, a) : cmp(a, b);
#ifdef LOCAL
        if (r && (desc ? cmp(a, b) : cmp(b, a))) fail(\"sort `cmp:` comparator is not a strict weak order: both cmp(a, b) and cmp(b, a) are true\");
#endif
        return r;
    };
#ifdef LOCAL
    for (const auto& x : seq) {
        if (cmp(x, x)) fail(\"sort `cmp:` comparator is not a strict weak order: cmp(x, x) is true\");
    }
#endif
    std::stable_sort(seq.begin(), seq.end(), lt);
#ifdef LOCAL
    for (size_t i = 1; i < seq.size(); i++) {
        if (lt(seq[i], seq[i - 1]) || lt(seq[i], seq[0]) || lt(seq.back(), seq[i - 1])) {
            fail(\"sort `cmp:` comparator is not a strict weak order: result is not ordered\");
        }
    }
#endif
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
        name: "range_at",
        needs: &["fail", "range"],
        code: "\
template <class T> T range_at(const Range<T>& r, ll i) { if (i < 0 || i >= r.len()) fail(\"index out of range\"); return r[i]; }
template <class T> ll key_pos(const Range<T>& r, ll k) { ll i = k - (ll)r.lo, n = r.len(); if (i < 0 || i >= n) key_fail(k, (ll)r.lo, n); return i; }
",
    },
    Piece {
        name: "first_val",
        needs: &["fail"],
        code: "\
template <class T> T first_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
inline u8 first_val(const std::string& s) { if (s.empty()) fail(\"the string is empty\"); return (u8)s.front(); }
",
    },
    Piece {
        name: "last_val",
        needs: &["fail"],
        code: "\
template <class T> T last_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
inline u8 last_val(const std::string& s) { if (s.empty()) fail(\"the string is empty\"); return (u8)s.back(); }
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
        name: "map_lend",
        needs: &[],
        code: "\
template <class V> struct Lent {
    const V* p;
    bool has_value() const { return p != nullptr; }
    explicit operator bool() const { return p != nullptr; }
    const V& operator*() const { return *p; }
    const V* operator->() const { return p; }
    operator std::optional<V>() const { return p ? std::optional<V>(*p) : std::nullopt; }
};
template <class M, class Q> Lent<typename M::mapped_type> map_lend(const M& m, const Q& k) {
    auto it = m.find(k); return {it == m.end() ? nullptr : &it->second};
}
",
    },
    Piece {
        name: "map_or",
        needs: &[],
        code: "\
template <class M, class Q> const typename M::mapped_type& map_or(const M& m, const Q& k, const typename M::mapped_type& d) {
    auto it = m.find(k); return it == m.end() ? d : it->second;
}
",
    },
    Piece {
        name: "map_move_or",
        needs: &[],
        code: "\
template <class M, class Q> typename M::mapped_type map_move_or(M& m, const Q& k, typename M::mapped_type d) {
    auto it = m.find(k); if (it == m.end()) return d; return std::move(it->second);
}
",
    },
    Piece {
        name: "map_take",
        needs: &[],
        code: "\
template <class M, class Q> auto map_take(M& m, const Q& k) -> std::optional<typename M::mapped_type> {
    auto it = m.find(k); if (it == m.end()) return std::nullopt;
    auto v = std::move(it->second); m.erase(it); return v;
}
",
    },
    Piece {
        name: "interval_assign",
        needs: &["fail", "range"],
        code: "\
template <class M, class Pos, class V, class F>
void interval_assign(M& segs, const Range<Pos>& r, const V& val, F&& on_old) {
    if (r.lo > r.hi) fail(\"the range runs backwards: its end is before its start\");
    if (r.lo == r.hi) return;
    ll lo = (ll)r.lo, hi = (ll)r.hi;
    auto it = segs.upper_bound(lo);
    if (it == segs.begin()) return;
    auto lit = std::prev(it);
    ll sl = lit->first;
    ll seg_hi = std::get<0>(lit->second);
    V cur_val = std::get<1>(lit->second);
    if (seg_hi >= hi) {
        on_old(r, cur_val);
        if (cur_val == val) return;
        ll new_hi = hi;
        if (seg_hi > hi) {
            it = segs.emplace_hint(it, hi, std::tuple<ll, V>{seg_hi, cur_val});
        } else if (it != segs.end() && it->first == hi && std::get<1>(it->second) == val) {
            new_hi = std::get<0>(it->second);
            it = segs.erase(it);
        }
        if (sl < lo) {
            std::get<0>(lit->second) = lo;
            segs.emplace_hint(it, lo, std::tuple<ll, V>{new_hi, val});
        } else if (lit != segs.begin() && std::get<1>(std::prev(lit)->second) == val) {
            std::get<0>(std::prev(lit)->second) = new_hi;
            segs.erase(lit);
        } else {
            lit->second = std::tuple<ll, V>{new_hi, val};
        }
        return;
    }
    on_old(Range<Pos>{(Pos)lo, (Pos)seg_hi}, cur_val);
    auto left_target = segs.end();
    if (sl < lo) {
        if (cur_val == val) left_target = lit;
        else std::get<0>(lit->second) = lo;
    } else if (lit != segs.begin() && std::get<1>(std::prev(lit)->second) == val) {
        left_target = std::prev(lit);
        segs.erase(lit);
    } else {
        left_target = lit;
    }
    ll new_hi = hi;
    while (it != segs.end() && it->first < hi) {
        ll pos = it->first;
        ll ohi = std::get<0>(it->second);
        V ov = std::move(std::get<1>(it->second));
        if (ohi <= hi) {
            on_old(Range<Pos>{(Pos)pos, (Pos)ohi}, ov);
            it = segs.erase(it);
        } else {
            on_old(Range<Pos>{(Pos)pos, (Pos)hi}, ov);
            if (ov == val) {
                new_hi = ohi;
                it = segs.erase(it);
            } else {
                it = segs.erase(it);
                it = segs.emplace_hint(it, hi, std::tuple<ll, V>{ohi, std::move(ov)});
            }
            break;
        }
    }
    if (it != segs.end() && it->first == new_hi && std::get<1>(it->second) == val) {
        new_hi = std::get<0>(it->second);
        it = segs.erase(it);
    }
    if (left_target != segs.end()) left_target->second = std::tuple<ll, V>{new_hi, val};
    else segs.emplace_hint(it, lo, std::tuple<ll, V>{new_hi, val});
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
    Piece {
        name: "str_view",
        needs: &["fail"],
        code: "\
// O(1) read-only slice/reverse view into a `Str` (`std::string_view` + reverse bit).
struct StrView {
    using value_type = u8;
    std::string_view sv;
    bool rev = false;
    ll k0 = 0;
    StrView() = default;
    StrView(const std::string& s, bool r = false) noexcept : sv(s), rev(r) {}
    StrView(std::string_view v, bool r = false, ll k = 0) noexcept : sv(v), rev(r), k0(k) {}
    void cpl_str_view_tag() const {}
    size_t size() const noexcept { return sv.size(); }
    bool empty() const noexcept { return sv.empty(); }
    u8 operator[](size_t i) const noexcept { return (u8)(rev ? sv[sv.size() - 1 - i] : sv[i]); }
    u8 front() const noexcept { return (*this)[0]; }
    u8 back() const noexcept { return (*this)[sv.size() - 1]; }
    StrView slice(ll lo, ll hi) const {
        if (lo < 0 || hi < lo || (size_t)hi > sv.size()) fail(\"slice outside the string\");
        size_t n = (size_t)(hi - lo), off = rev ? (sv.size() - (size_t)hi) : (size_t)lo;
        return StrView(sv.substr(off, n), rev, k0 + (ll)off);
    }
    StrView reversed() const noexcept { return StrView(sv, !rev, k0); }
    std::string to_str() const { return !rev ? std::string(sv) : std::string(sv.rbegin(), sv.rend()); }
    struct Iter {
        const char* d; size_t i, n; bool rev;
        u8 operator*() const noexcept { return (u8)(rev ? d[n - 1 - i] : d[i]); }
        Iter& operator++() noexcept { ++i; return *this; }
        bool operator!=(const Iter& o) const noexcept { return i != o.i; }
        bool operator==(const Iter& o) const noexcept { return i == o.i; }
    };
    Iter begin() const noexcept { return {sv.data(), 0, sv.size(), rev}; }
    Iter end() const noexcept { return {sv.data(), sv.size(), sv.size(), rev}; }
    Iter rbegin() const noexcept { return reversed().begin(); }
    Iter rend() const noexcept { return reversed().end(); }
    int cmp(StrView o) const noexcept {
        if (!rev && !o.rev) return sv.compare(o.sv);
        size_t n = sv.size() < o.sv.size() ? sv.size() : o.sv.size();
        for (size_t i = 0; i < n; i++) { u8 a = (*this)[i], b = o[i]; if (a != b) return a < b ? -1 : 1; }
        return sv.size() < o.sv.size() ? -1 : (sv.size() > o.sv.size() ? 1 : 0);
    }
    bool contains_char(u8 c) const noexcept { return sv.find((char)c) != std::string_view::npos; }
    bool contains_str(StrView p) const { return p.empty() || ((!rev && !p.rev) ? sv.find(p.sv) != std::string_view::npos : to_str().find(p.to_str()) != std::string::npos); }
    friend bool operator==(StrView a, StrView b) noexcept { return a.size() == b.size() && a.cmp(b) == 0; }
    friend bool operator!=(StrView a, StrView b) noexcept { return !(a == b); }
    friend bool operator<(StrView a, StrView b) noexcept { return a.cmp(b) < 0; }
    friend bool operator<=(StrView a, StrView b) noexcept { return a.cmp(b) <= 0; }
    friend bool operator>(StrView a, StrView b) noexcept { return a.cmp(b) > 0; }
    friend bool operator>=(StrView a, StrView b) noexcept { return a.cmp(b) >= 0; }
};
inline StrView str_slice(StrView s, ll lo, ll hi) { return s.slice(lo, hi); }
inline StrView str_rev(StrView s) noexcept { return s.reversed(); }
inline ll key_lo(StrView s) noexcept { return s.k0; }
inline ll key_pos(StrView s, ll k) { ll i = k - s.k0, n = (ll)s.size(); if (i < 0 || i >= n) key_fail(k, s.k0, n); return s.rev ? n - 1 - i : i; }
inline ll pos_key(StrView s, ll p) noexcept { return s.k0 + (s.rev ? (ll)s.size() - 1 - p : p); }
inline StrView key_slice(StrView s, ll lo, ll hi) { if (lo < s.k0 || hi < lo || hi > s.k0 + (ll)s.size()) fail(\"slice outside the string's keys\"); return StrView(s.sv.substr((size_t)(lo - s.k0), (size_t)(hi - lo)), s.rev, lo); }
inline std::string to_str(StrView s) { return s.to_str(); }
inline void append_str_view(std::string& o, StrView v) { if (!v.rev) o.append(v.sv.data(), v.sv.size()); else for (size_t i = v.sv.size(); i-- > 0;) o.push_back(v.sv[i]); }
inline std::string str_concat(StrView a, StrView b) { std::string o; o.reserve(a.size() + b.size()); append_str_view(o, a); append_str_view(o, b); return o; }
inline u8 at(StrView s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
inline std::optional<u8> first(StrView s) { return s.empty() ? std::optional<u8>() : std::optional<u8>(s.front()); }
inline std::optional<u8> last(StrView s) { return s.empty() ? std::optional<u8>() : std::optional<u8>(s.back()); }
inline std::vector<u8> str_to_list(StrView s) { std::vector<u8> v; v.reserve(s.size()); for (u8 c : s) v.push_back(c); return v; }
inline ll parse_int(StrView s) { return std::stoll(s.to_str()); }
",
    },
    Piece {
        name: "put_str_view",
        needs: &["out", "str_view"],
        code: "\
inline void put_str(StrView s) { append_str_view(out, s); }
",
    },
    Piece {
        name: "list_view",
        needs: &["fail"],
        code: "\
// O(1) read-only slice/reverse view into a `List<T>` (pointer + length + reverse bit).
template <class T> struct ListView {
    using value_type = T;
    const T* ptr = nullptr;
    size_t n = 0;
    bool rev = false;
    ll k0 = 0;
    ListView() = default;
    ListView(const std::vector<T>& v, bool r = false) noexcept : ptr(v.data()), n(v.size()), rev(r) {}
    ListView(const T* p, size_t sz, bool r = false, ll k = 0) noexcept : ptr(p), n(sz), rev(r), k0(k) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    const T& operator[](size_t i) const noexcept { return rev ? ptr[n - 1 - i] : ptr[i]; }
    const T& front() const noexcept { return (*this)[0]; }
    const T& back() const noexcept { return (*this)[n - 1]; }
    ListView reversed() const noexcept { return ListView(ptr, n, !rev, k0); }
    struct Iter {
        const T* d; size_t i, n; bool rev;
        const T& operator*() const noexcept { return rev ? d[n - 1 - i] : d[i]; }
        Iter& operator++() noexcept { ++i; return *this; }
        bool operator!=(const Iter& o) const noexcept { return i != o.i; }
        bool operator==(const Iter& o) const noexcept { return i == o.i; }
    };
    Iter begin() const noexcept { return {ptr, 0, n, rev}; }
    Iter end() const noexcept { return {ptr, n, n, rev}; }
    Iter rbegin() const noexcept { return reversed().begin(); }
    Iter rend() const noexcept { return reversed().end(); }
    int cmp(ListView o) const {
        size_t m = n < o.n ? n : o.n;
        for (size_t i = 0; i < m; i++) { const T& a = (*this)[i], &b = o[i]; if (a < b) return -1; if (b < a) return 1; }
        return n < o.n ? -1 : (n > o.n ? 1 : 0);
    }
    friend bool operator==(ListView a, ListView b) { return a.size() == b.size() && a.cmp(b) == 0; }
    friend bool operator!=(ListView a, ListView b) { return !(a == b); }
    friend bool operator<(ListView a, ListView b) { return a.cmp(b) < 0; }
    friend bool operator<=(ListView a, ListView b) { return a.cmp(b) <= 0; }
    friend bool operator>(ListView a, ListView b) { return a.cmp(b) > 0; }
    friend bool operator>=(ListView a, ListView b) { return a.cmp(b) >= 0; }
};
template <class T> ListView<T> list_slice(ListView<T> v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); size_t off = v.rev ? v.n - (size_t)hi : (size_t)lo; return ListView<T>(v.ptr + off, (size_t)(hi - lo), v.rev, v.k0 + (ll)off); }
template <class T> ll key_lo(ListView<T> v) noexcept { return v.k0; }
template <class T> ll key_pos(ListView<T> v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return v.rev ? n - 1 - i : i; }
template <class T> ll pos_key(ListView<T> v, ll p) noexcept { return v.k0 + (v.rev ? (ll)v.n - 1 - p : p); }
template <class T> ListView<T> key_slice(ListView<T> v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (lo - v.k0), (size_t)(hi - lo), v.rev, lo); }
template <class T> ListView<T> list_slice(const std::vector<T>& v, ll lo, ll hi) { return list_slice(ListView<T>(v), lo, hi); }
template <class T> ListView<T> key_slice(const std::vector<T>& v, ll lo, ll hi) { return key_slice(ListView<T>(v), lo, hi); }
template <class T> ListView<T> list_rev(const std::vector<T>& v) noexcept { return ListView<T>(v, true); }
template <class T> ListView<T> list_rev(ListView<T> v) noexcept { return v.reversed(); }
template <class T> std::vector<T> to_list(const std::vector<T>& v) { return v; }
template <class T> std::vector<T> to_list(ListView<T> v) { std::vector<T> o; o.reserve(v.n); append_list_view(o, v); return o; }
template <class T> void append_list_view(std::vector<T>& o, ListView<T> v) { if (!v.rev) o.insert(o.end(), v.ptr, v.ptr + v.n); else for (size_t i = v.n; i-- > 0;) o.push_back(v.ptr[i]); }
template <class T> std::vector<T> list_concat(ListView<T> a, ListView<T> b) { std::vector<T> o; o.reserve(a.size() + b.size()); append_list_view(o, a); append_list_view(o, b); return o; }
template <class T> std::vector<T> list_concat(ListView<T> a, const std::vector<T>& b) { return list_concat(a, ListView<T>(b)); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, ListView<T> b) { return list_concat(ListView<T>(a), b); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, const std::vector<T>& b) { return list_concat(ListView<T>(a), ListView<T>(b)); }
template <class T> const T& at(ListView<T> v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::optional<T> first(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <class T> std::vector<T> sorted(ListView<T> v) { auto o = to_list(v); std::sort(o.begin(), o.end()); return o; }
template <class T, class U> bool has(ListView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i] == x) return true; return false; }
inline std::string list_to_str(ListView<u8> v) { std::string s; s.reserve(v.size()); for (u8 c : v) s.push_back((char)c); return s; }
",
    },
    Piece {
        name: "rekey",
        needs: &["list_view"],
        code: "\
template <class T> ListView<T> rekey(ListView<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.n) fail(\"rekey needs as many keys as cells\"); return ListView<T>(v.ptr, v.n, v.rev, lo); }
template <class T> ListView<T> rekey(const std::vector<T>& v, ll lo, ll hi) { return rekey(ListView<T>(v), lo, hi); }
template <class T> struct Keyed : ListView<T> {
    std::vector<T> store;
    Keyed() = default;
    Keyed(std::vector<T> v, ll k) : store(std::move(v)) { this->ptr = store.data(); this->n = store.size(); this->k0 = k; }
    Keyed(const Keyed& o) : ListView<T>(o), store(o.store) { this->ptr = store.data(); }
    Keyed(Keyed&& o) noexcept : ListView<T>(o), store(std::move(o.store)) { this->ptr = store.data(); }
    Keyed& operator=(const Keyed& o) { ListView<T>::operator=(o); store = o.store; this->ptr = store.data(); return *this; }
    Keyed& operator=(Keyed&& o) noexcept { ListView<T>::operator=(o); store = std::move(o.store); this->ptr = store.data(); return *this; }
};
template <class T> Keyed<T> rekey_own(std::vector<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.size()) fail(\"rekey needs as many keys as cells\"); return Keyed<T>(std::move(v), lo); }
",
    },
    Piece {
        name: "mut_view",
        needs: &["fail"],
        code: "\
// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T` (pointer + length).
template <class T> struct MutView {
    using value_type = T;
    T* ptr = nullptr;
    size_t n = 0;
    ll k0 = 0;
    MutView() = default;
    MutView(T* p, size_t sz, ll k = 0) noexcept : ptr(p), n(sz), k0(k) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    T* data() noexcept { return ptr; }
    const T* data() const noexcept { return ptr; }
    T* begin() noexcept { return ptr; }
    T* end() noexcept { return ptr + n; }
    const T* begin() const noexcept { return ptr; }
    const T* end() const noexcept { return ptr + n; }
    T& operator[](size_t i) noexcept { return ptr[i]; }
    const T& operator[](size_t i) const noexcept { return ptr[i]; }
    T& front() noexcept { return ptr[0]; }
    const T& front() const noexcept { return ptr[0]; }
    T& back() noexcept { return ptr[n - 1]; }
    const T& back() const noexcept { return ptr[n - 1]; }
};
template <class T> MutView<T> mut_slice(std::vector<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.size()) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T, size_t N> MutView<T> mut_slice(std::array<T, N>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > N) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T> MutView<T> mut_slice(MutView<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); return MutView<T>(v.ptr + (size_t)lo, (size_t)(hi - lo), v.k0 + lo); }
template <class T> ll key_lo(const MutView<T>& v) noexcept { return v.k0; }
template <class T> ll key_pos(const MutView<T>& v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return i; }
template <class T> ll pos_key(const MutView<T>& v, ll p) noexcept { return v.k0 + p; }
template <class T> ListView<T> key_slice(const MutView<T>& v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (lo - v.k0), (size_t)(hi - lo), false, lo); }
template <class T> T& at(MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> const T& at(const MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::vector<T> to_list(MutView<T> v) { return std::vector<T>(v.ptr, v.ptr + v.n); }
template <class T> std::optional<T> first(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <class T> std::vector<T> sorted(MutView<T> v) { auto o = to_list(v); std::sort(o.begin(), o.end()); return o; }
template <class T, class U> bool has(MutView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i] == x) return true; return false; }
",
    },
    Piece {
        name: "list_concat_at",
        needs: &["at"],
        code: "\
template <class A, class B> auto list_concat_at(const A& a, const B& b, ll i) -> decltype(at(a, i)) { if (i < 0) fail(\"index outside the list\"); if ((size_t)i < a.size()) return a[(size_t)i]; return at(b, i - (ll)a.size()); }
",
    },
    Piece {
        name: "rec",
        needs: &[],
        code: "\
// A recursive local closure, passed itself first.
template <class F> struct Rec { F f; template <class... A> decltype(auto) operator()(A&&... a) { return f(*this, (A&&)a...); } };
template <class F> Rec<F> rec(F f) { return {f}; }
",
    },
    Piece {
        name: "lazy",
        needs: &[],
        code: "\
// A row a loop reaches by indices it never changes, found on first use.
template <class F> struct Lazy { F f; decltype(f()) p = 0; auto& operator()() { return *(p ? p : p = f()); } };
template <class F> Lazy<F> lazy(F f) { return {f}; }
",
    },
];

/// The one-line fns of the runtime in `file` whose name nothing else in
/// the file mentions, after the unused ones are gone. Operators, which
/// are called without their name, always stay.
pub(crate) fn unused_lines(file: &str) -> Vec<&str> {
    let Some(start) = file.find("namespace cpl {\n") else {
        return Vec::new();
    };
    let end = file[start..]
        .find("}  // namespace cpl\n")
        .map_or(file.len(), |at| start + at);
    let mut defs: Vec<(&str, &str)> = file[start..end]
        .lines()
        .filter_map(|line| Some((one_line_fn(line)?, line)))
        .collect();
    let mut unused = Vec::new();
    loop {
        let dead: Vec<&str> = defs
            .iter()
            .map(|&(name, _)| name)
            .filter(|&name| {
                !file.lines().any(|line| {
                    !unused.contains(&line)
                        && one_line_fn(line) != Some(name)
                        && mentions(line, name)
                })
            })
            .collect();
        if dead.is_empty() {
            return unused;
        }
        defs.retain(|&(name, line)| {
            !dead.contains(&name) || {
                unused.push(line);
                false
            }
        });
    }
}

fn word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The name a whole fn written on one line defines, as in
/// `template <class T> T last_val(ListView<T> v) { ... }`.
fn one_line_fn(line: &str) -> Option<&str> {
    let mut head = line.strip_prefix("inline ").unwrap_or(line).as_bytes();
    if !line.ends_with('}') || head.len() == line.len() && !line.starts_with("template <") {
        return None;
    }
    if head.len() == line.len() {
        let mut depth = 0;
        let close = head.iter().position(|&c| {
            depth += (c == b'<') as i32 - (c == b'>') as i32;
            c == b'>' && depth == 0
        })?;
        head = head[close + 1..].trim_ascii_start();
    }
    let paren = head.iter().position(|&c| c == b'(')?;
    let before = &head[..paren];
    let at = before.iter().rposition(|&c| !word(c))? + 1;
    let kind = before.starts_with(b"struct ")
        || before.starts_with(b"class ")
        || before.starts_with(b"using ");
    let name = std::str::from_utf8(&before[at..]).ok()?;
    (!kind && !name.is_empty() && !before.windows(8).any(|w| w == b"operator")).then_some(name)
}

/// Whether `line` names the free fn `name`: a comment line does not call
/// it, and `x.first` and `p->first` are members, which it never is.
fn mentions(line: &str, name: &str) -> bool {
    if line.trim_start().starts_with("//") {
        return false;
    }
    let (l, n) = (line.as_bytes(), name.as_bytes());
    (0..=l.len().saturating_sub(n.len())).any(|at| {
        let member = l[..at].ends_with(b".") || l[..at].ends_with(b"->");
        l[at..].starts_with(n)
            && (at == 0 || !word(l[at - 1]))
            && !member
            && l.get(at + n.len()).is_none_or(|&c| !word(c))
    })
}
