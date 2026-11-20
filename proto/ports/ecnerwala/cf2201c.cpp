// cf2201c.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2201 C: linear modular DP counting non-empty balanced-parenthesis
//     // subsequences whose prefix balance stays >= 2 at open-bracket extensions.
//     const MOD = 998244353
//     type Z = Mod<MOD>
//
//     @!multitest
//     let n: Int = read()
//     let s: Str = read()
//
//     var ways_any: Z = 0
//     var ways_open: Z = 0
//     var ways_close: Z = 0
//     var pref = 0
//     var ans: Z = 0
//     for i in 0..n:
//         ans += 1
//         if s[i] == '(':
//             ans += ways_any
//             ways_open += ways_open + ways_close + 1
//             pref += 1
//         else:
//             ans += ways_open + ways_close
//             ways_close += ways_open + ways_close + 1
//             pref -= 1
//         if pref < 2: ways_open = 0
//         ways_any = ways_any * 2 + 1
//     print(ans)

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
inline std::string read_str() { skip_space(); size_t start = in_at; while (in_at < in.size() && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
inline ll modp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    ll r = a % b; return r + ((r >> 63) & b);
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
struct Mod_RI0_2 {
    ll v = 0;
    auto cpl_tie() const { return std::tie(v); }
};

ll MOD = 0;
ll n = 0;
std::string s;
Mod_RI0_2 ways_any;
Mod_RI0_2 ways_open;
Mod_RI0_2 ways_close;
ll pref = 0;
Mod_RI0_2 ans;

Mod_RI0_2 Mod_RI0_2_new(ll x);
Mod_RI0_2 Mod_RI0_2_add(const Mod_RI0_2& a, const Mod_RI0_2& b);
Mod_RI0_2 Mod_RI0_2_mul(const Mod_RI0_2& a, const Mod_RI0_2& b);
std::string Mod_RI0_2_show(const Mod_RI0_2& a);
// Prints a `Mod_RI0_2`.
void show_1(const Mod_RI0_2& v, int) {
    cpl::put_str(Mod_RI0_2_show(v));
}


Mod_RI0_2 Mod_RI0_2_new(ll x) {
    return Mod_RI0_2{cpl::modp(x, MOD)};
}

Mod_RI0_2 Mod_RI0_2_add(const Mod_RI0_2& a, const Mod_RI0_2& b) {
    return Mod_RI0_2{a.v + b.v >= MOD ? a.v + b.v - MOD : a.v + b.v};
}

Mod_RI0_2 Mod_RI0_2_mul(const Mod_RI0_2& a, const Mod_RI0_2& b) {
    return Mod_RI0_2{cpl::modp(a.v * b.v, MOD)};
}

std::string Mod_RI0_2_show(const Mod_RI0_2& a) {
    return [&]() -> std::string {
        cpl::Capture t0;
        cpl::put_int(a.v);
        return t0.take();
    }();
}

bool setup() {
    MOD = 998244353;
    return true;
}

void solve() {
    n = cpl::read_int();
    s = cpl::read_str();
    ways_any = Mod_RI0_2_new(0);
    ways_open = Mod_RI0_2_new(0);
    ways_close = Mod_RI0_2_new(0);
    pref = 0;
    ans = Mod_RI0_2_new(0);
    for (ll i = 0; i < n; i++) {
        ans = ::prog::Mod_RI0_2_add(ans, Mod_RI0_2_new(1));
        if ((cpl::u8)s[i] == '(') {
            ans = ::prog::Mod_RI0_2_add(ans, ways_any);
            ways_open = ::prog::Mod_RI0_2_add(ways_open, ::prog::Mod_RI0_2_add(::prog::Mod_RI0_2_add(ways_open, ways_close), Mod_RI0_2_new(1)));
            pref += 1;
        } else {
            ans = ::prog::Mod_RI0_2_add(ans, ::prog::Mod_RI0_2_add(ways_open, ways_close));
            ways_close = ::prog::Mod_RI0_2_add(ways_close, ::prog::Mod_RI0_2_add(::prog::Mod_RI0_2_add(ways_open, ways_close), Mod_RI0_2_new(1)));
            pref -= 1;
        }
        if (pref < 2) {
            ways_open = Mod_RI0_2_new(0);
        }
        ways_any = ::prog::Mod_RI0_2_add(::prog::Mod_RI0_2_mul(ways_any, Mod_RI0_2_new(2)), Mod_RI0_2_new(1));
    }
    show_1(ans, 6); cpl::put_line();
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
