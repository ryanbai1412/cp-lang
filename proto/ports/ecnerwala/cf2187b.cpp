// cf2187b.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2187 B: bitwise top-down DP over the 4 sign combinations of (p - x, q - y)
//     // minimizing |p - x| + |q - y| subject to p & q == 0.
//     @!multitest
//     let (x, y): (Int, Int) = read()
//     var ans: ?(Int, Int, Int) = none
//
//     for dx in 0..2:
//         for dy in 0..2:
//             var dp: [0..2, 0..2]?(Int, Int, Int) = none
//             dp[0, 0] = (0, 0, 0)
//             for b in (0..=30).rev:
//                 let bx = Int(x.bit(b))
//                 let by = Int(y.bit(b))
//                 var ndp: [0..2, 0..2]?(Int, Int, Int) = none
//                 for cx in 0..2:
//                     for cy in 0..2:
//                         if let (w, px, py) = dp[cx, cy]:
//                             for vx in 0..2:
//                                 for vy in 0..2:
//                                     if vx == 1 and vy == 1: continue
//                                     if cx == 0 and vx != bx and Int(vx > bx) != dx: continue
//                                     if cy == 0 and vy != by and Int(vy > by) != dy: continue
//                                     let nx = Int(cx == 1 or vx != bx)
//                                     let ny = Int(cy == 1 or vy != by)
//                                     let nw = w * 2 + (if dx == 1: vx - bx else: bx - vx) + (if dy == 1: vy - by else: by - vy)
//                                     let cand = (nw, px * 2 + vx, py * 2 + vy)
//                                     if let prev = ndp[nx, ny]:
//                                         if nw < prev.0: ndp[nx, ny] = cand
//                                     else:
//                                         ndp[nx, ny] = cand
//                 dp = ndp
//             for cx in 0..2:
//                 for cy in 0..2:
//                     if let v = dp[cx, cy]:
//                         if let cur = ans:
//                             if v.0 < cur.0: ans = v
//                         else:
//                             ans = v
//
//     let (_, p, q) = ans!
//     print(p, q)

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
[[noreturn]] inline void fail(const char* why) { flush(); fputs(why, stderr); fputc('\n', stderr); exit(3); }
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
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return std::move(*o);
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

struct m1_Sieve {
    std::vector<ll> spf;
    std::vector<ll> primes;
    auto cpl_tie() const { return std::tie(spf, primes); }
};

ll x = 0;
ll y = 0;
std::optional<std::tuple<ll, ll, ll>> ans;
ll p = 0;
ll q = 0;

bool m1_bit(ll self, ll i);

bool m1_bit(ll self, ll i) {
    return i < 64 && (self >> i & 1) == 1;
}

bool setup() {
    return true;
}

void solve() {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    x = std::get<0>(t0);
    y = std::get<1>(t0);
    ans = std::optional<std::tuple<ll, ll, ll>>();
    for (ll dx = 0; dx < 2; dx++) {
        for (ll dy = 0; dy < 2; dy++) {
            cpl::Grid<std::optional<std::tuple<ll, ll, ll>>, 2> dp = cpl::grid<std::optional<std::tuple<ll, ll, ll>>, 2>({2, 2}, std::optional<std::tuple<ll, ll, ll>>());
            dp(0, 0) = std::optional<std::tuple<ll, ll, ll>>(std::tuple<ll, ll, ll>{0, 0, 0});
            for (ll b = 31 - 1; b >= 0; b--) {
                ll bx = m1_bit(x, b) ? 1 : 0;
                ll by = m1_bit(y, b) ? 1 : 0;
                cpl::Grid<std::optional<std::tuple<ll, ll, ll>>, 2> ndp = cpl::grid<std::optional<std::tuple<ll, ll, ll>>, 2>({2, 2}, std::optional<std::tuple<ll, ll, ll>>());
                for (ll cx = 0; cx < 2; cx++) {
                    for (ll cy = 0; cy < 2; cy++) {
                        std::optional<std::tuple<ll, ll, ll>> t1 = dp(cx, cy);
                        if (t1.has_value()) {
                            std::tuple<ll, ll, ll> t2 = *t1;
                            ll w = std::get<0>(std::move(t2));
                            ll px = std::get<1>(std::move(t2));
                            ll py = std::get<2>(std::move(t2));
                            for (ll vx = 0; vx < 2; vx++) {
                                for (ll vy = 0; vy < 2; vy++) {
                                    if (vx == 1 && vy == 1) {
                                        continue;
                                    }
                                    if (cx == 0 && vx != bx && (vx > bx ? 1 : 0) != dx) {
                                        continue;
                                    }
                                    if (cy == 0 && vy != by && (vy > by ? 1 : 0) != dy) {
                                        continue;
                                    }
                                    ll nx = cx == 1 || vx != bx ? 1 : 0;
                                    ll ny = cy == 1 || vy != by ? 1 : 0;
                                    ll nw = w * 2 + (dx == 1 ? vx - bx : bx - vx) + (dy == 1 ? vy - by : by - vy);
                                    std::tuple<ll, ll, ll> cand = std::tuple<ll, ll, ll>{nw, px * 2 + vx, py * 2 + vy};
                                    std::optional<std::tuple<ll, ll, ll>> t3 = ndp(nx, ny);
                                    if (t3.has_value()) {
                                        std::tuple<ll, ll, ll> prev = *t3;
                                        if (nw < std::get<0>(prev)) {
                                            ndp(nx, ny) = std::optional<std::tuple<ll, ll, ll>>(cand);
                                        }
                                    } else {
                                        ndp(nx, ny) = std::optional<std::tuple<ll, ll, ll>>(cand);
                                    }
                                }
                            }
                        }
                    }
                }
                dp = ndp;
            }
            for (ll cx_2 = 0; cx_2 < 2; cx_2++) {
                for (ll cy_2 = 0; cy_2 < 2; cy_2++) {
                    std::optional<std::tuple<ll, ll, ll>> t4 = dp(cx_2, cy_2);
                    if (t4.has_value()) {
                        std::tuple<ll, ll, ll> v = *t4;
                        std::optional<std::tuple<ll, ll, ll>> t5 = ans;
                        if (t5.has_value()) {
                            std::tuple<ll, ll, ll> cur = *t5;
                            if (std::get<0>(v) < std::get<0>(cur)) {
                                ans = std::optional<std::tuple<ll, ll, ll>>(v);
                            }
                        } else {
                            ans = std::optional<std::tuple<ll, ll, ll>>(v);
                        }
                    }
                }
            }
        }
    }
    std::tuple<ll, ll, ll> t6 = cpl::have(ans);
    std::get<0>(t6);
    p = std::get<1>(t6);
    q = std::get<2>(t6);
    cpl::put_int(p); cpl::put_char(' '); cpl::put_int(q); cpl::put_line();
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
