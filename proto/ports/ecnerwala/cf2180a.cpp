// cf2180a.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2180 A: largest value < L congruent to A modulo gcd(L, B).
//     @!multitest
//     let (l, a, b): (Int, Int, Int) = read()
//     let g = gcd(l, b)
//     print(l - g + a % g)

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
inline ll emod(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    if (b == -1) return 0;
    ll r = a % b, s = b >> 63; return r + ((r >> 63) & ((b ^ s) - s));
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

struct m1_Sieve {
    std::vector<ll> spf;
    std::vector<ll> primes;
    auto cpl_tie() const { return std::tie(spf, primes); }
};

ll l = 0;
ll a = 0;
ll b = 0;
ll g = 0;

ll m1_gcd(ll a_2, ll b_2);

ll m1_gcd(ll a_2, ll b_2) {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{std::abs(a_2), std::abs(b_2)};
    ll x = std::get<0>(std::move(t0));
    ll y = std::get<1>(std::move(t0));
    while (y != 0) {
        std::tuple<ll, ll> t1 = std::tuple<ll, ll>{y, cpl::emod(x, y)};
        x = std::get<0>(t1);
        y = std::get<1>(t1);
    }
    return x;
}

bool setup() {
    return true;
}

void solve() {
    std::tuple<ll, ll, ll> t0 = std::tuple<ll, ll, ll>{cpl::read_int(), cpl::read_int(), cpl::read_int()};
    l = std::get<0>(t0);
    a = std::get<1>(t0);
    b = std::get<2>(t0);
    g = m1_gcd(l, b);
    cpl::put_int(l - g + cpl::emod(a, g)); cpl::put_line();
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
