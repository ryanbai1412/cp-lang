// cf2201f1.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2201 F1: check whether the transposed Ferrers diagrams of the two
//     // row-frequency multisets match after each update via a double modular hash.
//     const M1 = 998244353
//     const M2 = 1_000_000_007
//     type Z1 = Mod<M1>
//     type Z2 = Mod<M2>
//
//     fn mix1(f: Int, g: Int) -> Z1:
//         let x: Z1 = f + 1315423911
//         let y: Z1 = g + 2654435761
//         let t = x * x + y * y * 31337 + x * y * 1000003 + 7
//         t * t * t
//
//     fn mix2(f: Int, g: Int) -> Z2:
//         let x: Z2 = f + 19260817
//         let y: Z2 = g + 1000000009
//         let t = x * x * 998244353 + y * y + x * y * 19260817 + 13
//         t * t * t
//
//     @!multitest
//     let (n, q): (Int, Int) = read()
//     var row_freq: [0..2, 0..n]Int = 0
//     var freq_freq: [0..2, 0..n]Int = 0
//     var h1: [0..2]Z1 = 0
//     var h2: [0..2]Z2 = 0
//
//     for _ in 0..q:
//         let (r0, r1): (Int, Int) = read()
//         let rs: [0..2]Int = [r0 - 1, r1 - 1]
//         for z in 0..2:
//             let r = rs[z]
//             let f = row_freq[z, r]
//             row_freq[z, r] = f + 1
//             let g = freq_freq[z, f]
//             freq_freq[z, f] = g + 1
//             let a = if z == 0: f else: g
//             let b = if z == 0: g else: f
//             h1[z] += mix1(a, b)
//             h2[z] += mix2(a, b)
//         print(if h1[0] == h1[1] and h2[0] == h2[1]: "YES" else: "NO")

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <optional>
#include <set>
#include <string>
#include <tuple>
#include <vector>

using ll = long long;

// What the program below leans on.
namespace cpl {
using u8 = unsigned char;
// Output is collected here and written once at the end.
std::string out;
inline void flush() { fwrite(out.data(), 1, out.size(), stdout); out.clear(); }
inline void put_char(u8 c) { out.push_back((char)c); }
inline void put_str(const std::string& s) { out += s; }
inline void put_str(const char* s) { out += s; }
inline void put_line() { out.push_back('\n'); if (out.size() >= (1u << 16)) flush(); }
inline void put_int(ll x) { char b[24]; out.append(b, (size_t)snprintf(b, sizeof b, "%lld", x)); }
// Collects what is printed while it lives: how a "{x}" string is built.
struct Capture {
    std::string saved;
    Capture() { saved.swap(out); }
    std::string take() { std::string text; text.swap(out); out.swap(saved); return text; }
};
// All of the input, read once.
std::string in;
size_t in_at = 0;
inline void read_all() { char b[1 << 16]; size_t n; while ((n = fread(b, 1, sizeof b, stdin)) > 0) in.append(b, n); }
inline bool is_space(char c) { return c == ' ' || c == '\n' || c == '\t' || c == '\r' || c == '\f'; }
// An interactive program reads a line when it needs one: the judge
// answers only after it has seen the query.
bool live = false;
inline bool more() { if (!live) return false; char b[1 << 12]; bool got = false; while (fgets(b, sizeof b, stdin)) { got = true; in += b; if (in.back() == '\n') break; } return got; }
inline void skip_space() { for (;;) { while (in_at < in.size() && is_space(in[in_at])) in_at++; if (in_at < in.size() || !more()) return; } }
inline ll read_int() {
    skip_space();
    bool negative = false;
    if (in_at < in.size() && (in[in_at] == '-' || in[in_at] == '+')) negative = in[in_at++] == '-';
    ll x = 0;  // built downward, so the smallest Int reads without overflow
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 - (in[in_at++] - '0');
    return negative ? x : -x;
}
inline ll modp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    ll r = a % b; return r + ((r >> 63) & b);
}
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
}  // namespace cpl

// The program.
namespace prog {
// A struct compares field by field, in order.
template <class T> auto operator==(const T& a, const T& b) -> decltype(a.cpl_tie() == b.cpl_tie()) { return a.cpl_tie() == b.cpl_tie(); }
template <class T> auto operator!=(const T& a, const T& b) -> decltype(a.cpl_tie() != b.cpl_tie()) { return a.cpl_tie() != b.cpl_tie(); }
template <class T> auto operator<(const T& a, const T& b) -> decltype(a.cpl_tie() < b.cpl_tie()) { return a.cpl_tie() < b.cpl_tie(); }
template <class T> auto operator<=(const T& a, const T& b) -> decltype(a.cpl_tie() <= b.cpl_tie()) { return a.cpl_tie() <= b.cpl_tie(); }
template <class T> auto operator>(const T& a, const T& b) -> decltype(a.cpl_tie() > b.cpl_tie()) { return a.cpl_tie() > b.cpl_tie(); }
template <class T> auto operator>=(const T& a, const T& b) -> decltype(a.cpl_tie() >= b.cpl_tie()) { return a.cpl_tie() >= b.cpl_tie(); }

struct m2_Word {
    std::string s;
    auto cpl_tie() const { return std::tie(s); }
};
struct m3_Sieve {
    std::vector<ll> spf;
    std::vector<ll> primes;
    auto cpl_tie() const { return std::tie(spf, primes); }
};
struct Mod_998244353_2 {
    ll v = 0;
    auto cpl_tie() const { return std::tie(v); }
};
struct Mod_1000000007_3 {
    ll v = 0;
    auto cpl_tie() const { return std::tie(v); }
};
struct Mod_RI0_4 {
    ll v = 0;
    auto cpl_tie() const { return std::tie(v); }
};
struct Mod_RI1_5 {
    ll v = 0;
    auto cpl_tie() const { return std::tie(v); }
};

ll M1 = 0;
ll M2 = 0;
ll n = 0;
ll q = 0;
cpl::Grid<ll, 2> row_freq;
cpl::Grid<ll, 2> freq_freq;
std::vector<Mod_RI0_4> h1;
std::vector<Mod_RI1_5> h2;

Mod_998244353_2 mix1(ll f, ll g);
Mod_1000000007_3 mix2(ll f, ll g);
std::string Mod_998244353_2_show(const Mod_998244353_2& a);
std::string Mod_1000000007_3_show(const Mod_1000000007_3& a);
Mod_RI0_4 Mod_RI0_4_new(ll x);
Mod_RI0_4 Mod_RI0_4_add(const Mod_RI0_4& a, const Mod_RI0_4& b);
Mod_RI0_4 Mod_RI0_4_mul(const Mod_RI0_4& a, const Mod_RI0_4& b);
std::string Mod_RI0_4_show(const Mod_RI0_4& a);
Mod_RI1_5 Mod_RI1_5_new(ll x);
Mod_RI1_5 Mod_RI1_5_add(const Mod_RI1_5& a, const Mod_RI1_5& b);
Mod_RI1_5 Mod_RI1_5_mul(const Mod_RI1_5& a, const Mod_RI1_5& b);
std::string Mod_RI1_5_show(const Mod_RI1_5& a);

Mod_998244353_2 mix1(ll f, ll g) {
    Mod_RI0_4 x = Mod_RI0_4_new(f + 1315423911);
    Mod_RI0_4 y = Mod_RI0_4_new(g + 2654435761LL);
    Mod_RI0_4 t = ::prog::Mod_RI0_4_add(::prog::Mod_RI0_4_add(::prog::Mod_RI0_4_add(::prog::Mod_RI0_4_mul(x, x), ::prog::Mod_RI0_4_mul(::prog::Mod_RI0_4_mul(y, y), Mod_RI0_4_new(31337))), ::prog::Mod_RI0_4_mul(::prog::Mod_RI0_4_mul(x, y), Mod_RI0_4_new(1000003))), Mod_RI0_4_new(7));
    Mod_RI0_4 t0 = ::prog::Mod_RI0_4_mul(::prog::Mod_RI0_4_mul(t, t), t);
    return Mod_998244353_2{t0.v};
}

Mod_1000000007_3 mix2(ll f, ll g) {
    Mod_RI1_5 x = Mod_RI1_5_new(f + 19260817);
    Mod_RI1_5 y = Mod_RI1_5_new(g + 1000000009);
    Mod_RI1_5 t = ::prog::Mod_RI1_5_add(::prog::Mod_RI1_5_add(::prog::Mod_RI1_5_add(::prog::Mod_RI1_5_mul(::prog::Mod_RI1_5_mul(x, x), Mod_RI1_5_new(998244353)), ::prog::Mod_RI1_5_mul(y, y)), ::prog::Mod_RI1_5_mul(::prog::Mod_RI1_5_mul(x, y), Mod_RI1_5_new(19260817))), Mod_RI1_5_new(13));
    Mod_RI1_5 t0 = ::prog::Mod_RI1_5_mul(::prog::Mod_RI1_5_mul(t, t), t);
    return Mod_1000000007_3{t0.v};
}

std::string Mod_998244353_2_show(const Mod_998244353_2& a) {
    return [&]() -> std::string {
        cpl::Capture t0;
        cpl::put_int(a.v);
        return t0.take();
    }();
}

std::string Mod_1000000007_3_show(const Mod_1000000007_3& a) {
    return [&]() -> std::string {
        cpl::Capture t0;
        cpl::put_int(a.v);
        return t0.take();
    }();
}

Mod_RI0_4 Mod_RI0_4_new(ll x) {
    return Mod_RI0_4{cpl::modp(x, M1)};
}

Mod_RI0_4 Mod_RI0_4_add(const Mod_RI0_4& a, const Mod_RI0_4& b) {
    return Mod_RI0_4{a.v + b.v >= M1 ? a.v + b.v - M1 : a.v + b.v};
}

Mod_RI0_4 Mod_RI0_4_mul(const Mod_RI0_4& a, const Mod_RI0_4& b) {
    return Mod_RI0_4{cpl::modp(a.v * b.v, M1)};
}

std::string Mod_RI0_4_show(const Mod_RI0_4& a) {
    return [&]() -> std::string {
        cpl::Capture t0;
        cpl::put_int(a.v);
        return t0.take();
    }();
}

Mod_RI1_5 Mod_RI1_5_new(ll x) {
    return Mod_RI1_5{cpl::modp(x, M2)};
}

Mod_RI1_5 Mod_RI1_5_add(const Mod_RI1_5& a, const Mod_RI1_5& b) {
    return Mod_RI1_5{a.v + b.v >= M2 ? a.v + b.v - M2 : a.v + b.v};
}

Mod_RI1_5 Mod_RI1_5_mul(const Mod_RI1_5& a, const Mod_RI1_5& b) {
    return Mod_RI1_5{cpl::modp(a.v * b.v, M2)};
}

std::string Mod_RI1_5_show(const Mod_RI1_5& a) {
    return [&]() -> std::string {
        cpl::Capture t0;
        cpl::put_int(a.v);
        return t0.take();
    }();
}

bool setup() {
    M1 = 998244353;
    M2 = 1000000007;
    return true;
}

void solve() {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    n = std::get<0>(t0);
    q = std::get<1>(t0);
    row_freq = cpl::grid<ll, 2>({2, n}, 0);
    freq_freq = cpl::grid<ll, 2>({2, n}, 0);
    h1 = std::vector<Mod_RI0_4>(2, Mod_RI0_4_new(0));
    h2 = std::vector<Mod_RI1_5>(2, Mod_RI1_5_new(0));
    for (ll t1 = 0; t1 < q; t1++) {
        std::tuple<ll, ll> t2 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
        ll r0 = std::get<0>(std::move(t2));
        ll r1 = std::get<1>(std::move(t2));
        std::vector<ll> t3(2, r0 - 1);
        t3[1] = r1 - 1;
        std::vector<ll> rs = t3;
        for (ll z = 0; z < 2; z++) {
            ll r = rs[z];
            ll f = row_freq(z, r);
            row_freq(z, r) = f + 1;
            ll g = freq_freq(z, f);
            freq_freq(z, f) = g + 1;
            ll a = z == 0 ? f : g;
            ll b = z == 0 ? g : f;
            h1[z] = ::prog::Mod_RI0_4_add(h1[z], [&]() -> Mod_RI0_4 {
                Mod_998244353_2 t4 = mix1(a, b);
                return Mod_RI0_4{t4.v};
            }());
            h2[z] = ::prog::Mod_RI1_5_add(h2[z], [&]() -> Mod_RI1_5 {
                Mod_1000000007_3 t5 = mix2(a, b);
                return Mod_RI1_5{t5.v};
            }());
        }
        cpl::put_str(h1[0] == h1[1] && h2[0] == h2[1] ? std::string("YES") : std::string("NO")); cpl::put_line();
    }
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
