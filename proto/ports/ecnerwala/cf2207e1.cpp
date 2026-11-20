// cf2207e1.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2207 E1: validate the non-increasing suffix-MEX bounds, then
//     // reconstruct a valid array backwards using a stack of available values.
//     @!multitest
//     let n: Int = read()
//     let a: [0..n]Int = read()
//
//     var good = true
//     for i in 0..n - 1:
//         if a[i] < a[i + 1]: good = false
//     for i in 0..n:
//         if a[i] < n - i - 1 or a[i] > n: good = false
//
//     if not good:
//         print("NO")
//     else:
//         var b: List<Int> = []
//         var stk: List<Int> = []
//         var prv = -1
//         for i in (0..=n).rev:
//             let cur = if i == 0: n else: a[i - 1]
//             while prv < cur:
//                 prv += 1
//                 stk.push(prv)
//             b.push(stk.pop()!)
//         b.reverse()
//         b.pop()
//         print("YES")
//         print(b)

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
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<ll> a;
bool good = false;

// Prints a `List<Int>`.
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
    for (auto& t0 : a) t0 = cpl::read_int();
    good = true;
    for (ll i = 0, i_end = n - 1; i < i_end; i++) {
        if (a[i] < a[i + 1]) {
            good = false;
        }
    }
    for (ll i_2 = 0; i_2 < n; i_2++) {
        if (a[i_2] < n - i_2 - 1 || a[i_2] > n) {
            good = false;
        }
    }
    if (!good) {
        cpl::put_str(std::string("NO")); cpl::put_line();
    } else {
        std::vector<ll> b = std::vector<ll>();
        std::vector<ll> stk = std::vector<ll>();
        ll prv = -1;
        for (ll i_3 = n + 1 - 1; i_3 >= 0; i_3--) {
            ll cur = i_3 == 0 ? n : a[i_3 - 1];
            while (prv < cur) {
                prv += 1;
                stk.push_back(prv);
            }
            b.push_back(cpl::have(cpl::pop(stk)));
        }
        std::reverse(b.begin(), b.end());
        cpl::pop(b);
        cpl::put_str(std::string("YES")); cpl::put_line();
        show_1(b, 6); cpl::put_line();
    }
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
