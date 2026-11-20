// cf2201b.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2201 B: construct a length-2N array containing each value 1..N twice
//     // with exactly K distinct adjacent pair sums when N <= K <= 2N - 1.
//     @!multitest
//     let (n_in, k): (Int, Int) = read()
//     if k < n_in or k > 2 * n_in - 1:
//         print("NO")
//     else:
//         print("YES")
//         let n2 = k - n_in + 1
//         var n = n_in
//         var out: List<Int> = []
//         while n > n2:
//             n -= 1
//             out.push(n + 1)
//             out.push(n + 1)
//         if n == 1:
//             out.push(1)
//             out.push(1)
//         else:
//             out.push(1)
//             out.push(2)
//             for i in 1..n - 1:
//                 out.push(i + 2)
//                 out.push(i)
//             out.push(n - 1)
//             out.push(n)
//         print(out)

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
ll n_in = 0;
ll k = 0;

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
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    n_in = std::get<0>(t0);
    k = std::get<1>(t0);
    if (k < n_in || k > 2 * n_in - 1) {
        cpl::put_str(std::string("NO")); cpl::put_line();
    } else {
        cpl::put_str(std::string("YES")); cpl::put_line();
        ll n2 = k - n_in + 1;
        ll n = n_in;
        std::vector<ll> out = std::vector<ll>();
        while (n > n2) {
            n -= 1;
            out.push_back(n + 1);
            out.push_back(n + 1);
        }
        if (n == 1) {
            out.push_back(1);
            out.push_back(1);
        } else {
            out.push_back(1);
            out.push_back(2);
            for (ll i = 1, i_end = n - 1; i < i_end; i++) {
                out.push_back(i + 2);
                out.push_back(i);
            }
            out.push_back(n - 1);
            out.push_back(n);
        }
        show_1(out, 6); cpl::put_line();
    }
}
}  // namespace prog

int main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
    return 0;
}
