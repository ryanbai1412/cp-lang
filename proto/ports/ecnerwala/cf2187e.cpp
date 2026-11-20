// cf2187e.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2187 E: DP over the number of carried keys (bounded by O(sqrt(max A + N))),
//     // updating waiting vs spending a key at each door and pruning unreachable key counts.
//     @!multitest
//     let n: Int = read()
//     let a: [0..n]Int = read()
//     let s: Str = read()
//
//     let t_inf = n + 1 + max(a)!
//     var dp: List<Int> = [0]
//     var ans: [0..n]Int = 0
//     for i in 0..n:
//         if s[i] == '1':
//             var ndp: List<Int> = [dp[0]]
//             for v in dp: ndp.push(v)
//             dp = ndp
//         let len = dp.len
//         for k in 0..len:
//             var best = max(a[i], dp[k]) + 1 + 2 * max(k - 1, 0)
//             if k + 1 < len:
//                 best min= dp[k + 1] + 1 + 2 * k
//             dp[k] = best
//         while dp.last! > t_inf: dp.pop()
//         ans[i] = dp[0]
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
inline std::string read_str() { skip_space(); size_t start = in_at; while (in_at < in.size() && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return std::move(*o);
}
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
template <class T> std::optional<T> last(const std::vector<T>& v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<ll> a;
std::string s;
ll t_inf = 0;
std::vector<ll> dp;
std::vector<ll> ans;

// Prints a `[Int]Int`.
void show_1(const std::vector<ll>& v, int) {
    for (size_t i = 0; i < v.size(); i++) {
        if (i > 0) cpl::put_char(' ');
        cpl::put_int(v[i]);
    }
}


bool setup() {
    return true;
}

void solve() {
    n = cpl::read_int();
    a.assign(n, ll());
    for (auto& t5 : a) t5 = cpl::read_int();
    s = cpl::read_str();
    t_inf = n + 1 + cpl::have([&]() -> std::optional<ll> {
        ll t1 = 0;
        bool t2 = false;
        for (ll t0 : a) {
            ll t3 = t0;
            if (t2) {
                t1 = std::max<ll>(t1, t3);
            } else {
                t1 = t3;
                t2 = true;
            }
        }
        std::optional<ll> t4 = t2 ? std::optional<ll>(t1) : std::optional<ll>();
        return t4;
    }());
    dp = std::vector<ll>{0};
    ans = std::vector<ll>(n, 0);
    for (ll i = 0; i < n; i++) {
        if ((cpl::u8)s[i] == '1') {
            std::vector<ll> ndp = std::vector<ll>{cpl::at(dp, 0)};
            for (ll v : dp) {
                ndp.push_back(v);
            }
            dp = ndp;
        }
        ll len = (ll)dp.size();
        for (ll k = 0; k < len; k++) {
            ll best = std::max<ll>(a[i], cpl::at(dp, k)) + 1 + 2 * std::max<ll>(k - 1, 0);
            if (k + 1 < len) {
                best = std::min<ll>(best, cpl::at(dp, k + 1) + 1 + 2 * k);
            }
            cpl::at(dp, k) = best;
        }
        while (cpl::have(cpl::last(dp)) > t_inf) {
            cpl::pop(dp);
        }
        ans[i] = cpl::at(dp, 0);
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
