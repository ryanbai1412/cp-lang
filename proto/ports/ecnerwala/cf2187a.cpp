// cf2187a.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2187 A: sort (value, index) pairs and minimize max distance to the
//     // global min/max over all elements not already in sorted position.
//     @!multitest
//     let n: Int = read()
//     let a: [0..n]Int = read()
//     let s = sorted([(a[i], i) for i in 0..n])
//     let lo = s[0].0
//     let hi = s.last!.0
//
//     var ans: ?Int = none
//     for i in 0..n:
//         if s[i].1 != i:
//             let d = max(s[i].0 - lo, hi - s[i].0)
//             ans = if let cur = ans: min(cur, d) else: d
//     print(ans ?? -1)

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
template <class T> std::optional<T> last(const std::vector<T>& v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> std::vector<T> sorted(std::vector<T> v) { std::sort(v.begin(), v.end()); return v; }
template <class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<ll> a;
std::vector<std::tuple<ll, ll>> s;
ll lo = 0;
ll hi = 0;
std::optional<ll> ans;


bool setup() {
    return true;
}

void solve() {
    n = cpl::read_int();
    a.assign(n, ll());
    for (auto& t2 : a) t2 = cpl::read_int();
    s = cpl::sorted([&]() -> std::vector<std::tuple<ll, ll>> {
        std::vector<std::tuple<ll, ll>> t0 = std::vector<std::tuple<ll, ll>>();
        for (ll i = 0; i < n; i++) {
            t0.push_back(std::tuple<ll, ll>{a[i], i});
        }
        return t0;
    }());
    lo = std::get<0>(cpl::at(s, 0));
    hi = std::get<0>(cpl::have(cpl::last(s)));
    ans = std::optional<ll>();
    for (ll i_2 = 0; i_2 < n; i_2++) {
        if (std::get<1>(cpl::at(s, i_2)) != i_2) {
            ll d = std::max<ll>(std::get<0>(cpl::at(s, i_2)) - lo, hi - std::get<0>(cpl::at(s, i_2)));
            ans = [&]() -> std::optional<ll> {
                std::optional<ll> t1 = ans;
                if (t1.has_value()) {
                    ll cur = *t1;
                    return std::optional<ll>(std::min<ll>(cur, d));
                } else {
                    return std::optional<ll>(d);
                }
            }();
        }
    }
    cpl::put_int(ans.has_value() ? *ans : -1); cpl::put_line();
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
