//! Text in and out: the output buffer, number printing, the judge's
//! input readers and the runtime-error exit.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
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
// The user line whose call entered the library code that fails (R09.4.5).
inline ll site = 0;
inline std::string site_line() { return site ? \" (line \" + std::to_string(site) + \")\" : std::string(); }
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
];
