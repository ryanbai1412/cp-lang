// cf2183b.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2183 B: compute the smallest missing non-negative integer below K - 1.
//     @!multitest
//     let (n, k): (Int, Int) = read()
//     let a: [0..n]Int = read()
//     var has: [0..=n]Bool = false
//     for i in 0..n: has[a[i]] = true
//     var ans = 0
//     while ans < k - 1 and has[ans]: ans += 1
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
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
ll k = 0;
std::vector<ll> a;
std::vector<char> has;
ll ans = 0;


bool setup() {
    return true;
}

void solve() {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    n = std::get<0>(t0);
    k = std::get<1>(t0);
    a.assign(n, ll());
    for (auto& t2 : a) t2 = cpl::read_int();
    has = std::vector<char>(n + 1, false);
    for (ll i = 0; i < n; i++) {
        ll t1 = a[i];
        has[t1] = true;
    }
    ans = 0;
    while (ans < k - 1 && (bool)has[ans]) {
        ans += 1;
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
