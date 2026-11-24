//! Arithmetic C++ spells differently: Euclidean division, 128-bit helpers,
//! float traps, checked ranges, shifts powers and a random source.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
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
        name: "iarith",
        needs: &["fail"],
        code: "\
#ifdef LOCAL
[[noreturn]] inline void overflow(ll a, char op, ll b) { char why[128]; snprintf(why, sizeof why, \"integer overflow: `%lld %c %lld` does not fit in an `Int` (64 bits)\", a, op, b); fail(why); }
[[noreturn]] inline void overflow(__int128, char op, __int128) { char why[128]; snprintf(why, sizeof why, \"integer overflow: the result of `%c` does not fit in an `I128` (128 bits)\", op); fail(why); }
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
        name: "ishl",
        needs: &["fail"],
        code: "\
// `a << b` for a shift amount already in 0..64: traps as `cpl run` does in a
// checked build, wraps in a judge build, and is never C++'s undefined shift.
inline ll ishl(ll a, ll b) {
#ifdef LOCAL
    char why[112];
    if (a < 0) { snprintf(why, sizeof why, \"`<<` of the negative number %lld\", a); fail(why); }
    if (b > 0 && a > (std::numeric_limits<ll>::max() >> b)) { snprintf(why, sizeof why, \"integer overflow: `%lld << %lld` does not fit in an `Int` (64 bits)\", a, b); fail(why); }
#endif
    return (ll)((unsigned long long)a << b);
}
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
];
