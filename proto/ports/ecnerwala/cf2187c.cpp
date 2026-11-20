// cf2187c.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2187 C: greedily take the farthest forward edge at each vertex,
//     // then merge depth-indexed (count, subtree-sum) tables bottom-up (small-to-large).
//     @!multitest
//     let (n, m): (Int, Int) = read()
//     var nxt = [i + 1 for i in 0..n]
//     for _ in 0..m:
//         let (u, v): (Int, Int) = read()
//         nxt[u - 1] max= v - 1
//
//     var sz: [0..n]Int = 0
//     var by_depth: List<List<(Int, Int)>> = List.filled(n, [])
//     var ans = 0
//     for i in 0..n:
//         sz[i] += 1
//         by_depth[i].push((1, sz[i]))
//         if i < n - 1:
//             let p = nxt[i]
//             sz[p] += sz[i]
//             if by_depth[p].len < by_depth[i].len:
//                 let tmp = by_depth[i].take()
//                 by_depth[i] = by_depth[p].take()
//                 by_depth[p] = tmp
//             let lp = by_depth[p].len
//             let li = by_depth[i].len
//             for l in 1..=li:
//                 let (af, as2) = by_depth[p][lp - l]
//                 let (bf, bs2) = by_depth[i][li - l]
//                 ans += (af * bs2 + bf * as2) * l
//                 by_depth[p][lp - l] = (af + bf, as2 + bs2)
//             by_depth[i] = []
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
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
ll m = 0;
std::vector<ll> nxt;
std::vector<ll> sz;
std::vector<std::vector<std::tuple<ll, ll>>> by_depth;
ll ans = 0;


bool setup() {
    return true;
}

void solve() {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    n = std::get<0>(t0);
    m = std::get<1>(t0);
    std::vector<ll> t1 = std::vector<ll>();
    for (ll i = 0; i < n; i++) {
        t1.push_back(i + 1);
    }
    nxt = t1;
    for (ll t2 = 0; t2 < m; t2++) {
        std::tuple<ll, ll> t3 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
        ll u = std::get<0>(std::move(t3));
        ll v = std::get<1>(std::move(t3));
        cpl::at(nxt, u - 1) = std::max<ll>(cpl::at(nxt, u - 1), v - 1);
    }
    sz = std::vector<ll>(n, 0);
    by_depth = std::vector<std::vector<std::tuple<ll, ll>>>((size_t)(n), std::vector<std::tuple<ll, ll>>());
    ans = 0;
    for (ll i_2 = 0; i_2 < n; i_2++) {
        sz[i_2] += 1;
        cpl::at(by_depth, i_2).push_back(std::tuple<ll, ll>{1, sz[i_2]});
        if (i_2 < n - 1) {
            ll p = cpl::at(nxt, i_2);
            sz[p] += sz[i_2];
            if ((ll)cpl::at(by_depth, p).size() < (ll)cpl::at(by_depth, i_2).size()) {
                std::vector<std::tuple<ll, ll>> tmp = std::exchange(cpl::at(by_depth, i_2), {});
                cpl::at(by_depth, i_2) = std::exchange(cpl::at(by_depth, p), {});
                cpl::at(by_depth, p) = tmp;
            }
            ll lp = (ll)cpl::at(by_depth, p).size();
            ll li = (ll)cpl::at(by_depth, i_2).size();
            for (ll l = 1, l_end = li + 1; l < l_end; l++) {
                std::tuple<ll, ll> t4 = cpl::at(cpl::at(by_depth, p), lp - l);
                ll af = std::get<0>(std::move(t4));
                ll as2 = std::get<1>(std::move(t4));
                std::tuple<ll, ll> t5 = cpl::at(cpl::at(by_depth, i_2), li - l);
                ll bf = std::get<0>(std::move(t5));
                ll bs2 = std::get<1>(std::move(t5));
                ans += (af * bs2 + bf * as2) * l;
                cpl::at(cpl::at(by_depth, p), lp - l) = std::tuple<ll, ll>{af + bf, as2 + bs2};
            }
            cpl::at(by_depth, i_2) = std::vector<std::tuple<ll, ll>>();
        }
    }
    cpl::put_int(ans); cpl::put_line();
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
