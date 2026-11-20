// cf2187g.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2187 G: find each node's nearest left/right children from the matrix S,
//     // then walk from index 0 following the unique valid parent link to assign 1..N.
//     @!multitest
//     let n: Int = read()
//     let s: [0..n]Str = read()
//
//     var ch0: [0..n]?Int = none
//     var ch1: [0..n]?Int = none
//     for i in 0..n:
//         for j in (0..i).rev:
//             if s[j][i] == '1': ch0[i] = j
//         for j in i + 1..n:
//             if s[j][i] == '1': ch1[i] = j
//
//     var ans: [0..n]Int = 0
//     var cur = 0
//     for v in 1..=n:
//         ans[cur] = v
//         var pa0: ?Int = none
//         var pa1: ?Int = none
//         for j in (0..cur).rev:
//             if s[cur][j] == '1': pa0 = j
//         for j in cur + 1..n:
//             if s[cur][j] == '1': pa1 = j
//         if let p0 = pa0 and ch1[p0] == some(cur):
//             cur = p0
//         elif let p1 = pa1:
//             cur = p1
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
inline std::string read_str() { skip_space(); size_t start = in_at; while (in_at < in.size() && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
// `?V` held in one scalar: `none` is a value of S that no V is stored as, so
// there is no flag beside the value. Everything else is std::optional's: `none`
// sorts first, and nothing converts to or from it without being asked.
template <class T, class S, S NONE> struct Niche {
    S raw = NONE;
    Niche() = default;
    explicit Niche(T x) : raw((S)x) {}
    explicit Niche(const std::optional<T>& o) : raw(o ? (S)*o : NONE) {}
    // A `List<Bool>` keeps a `char` per flag, so what it hands back is a `std::optional<char>`.
    template <class U> explicit Niche(const std::optional<U>& o) : raw(o ? (S)(T)*o : NONE) {}
    bool has_value() const { return raw != NONE; }
    explicit operator bool() const { return raw != NONE; }
    T operator*() const { return (T)raw; }
    template <class U = S, class = std::enable_if_t<std::is_same<U, T>::value>> T& operator*() { return raw; }
    explicit operator std::optional<T>() const { return raw != NONE ? std::optional<T>((T)raw) : std::nullopt; }
    friend bool operator==(Niche a, Niche b) { return a.raw == b.raw; }
    friend bool operator!=(Niche a, Niche b) { return a.raw != b.raw; }
    friend bool operator<(Niche a, Niche b) { return b.has_value() && (!a.has_value() || *a < *b); }
    friend bool operator>(Niche a, Niche b) { return b < a; }
    friend bool operator<=(Niche a, Niche b) { return !(b < a); }
    friend bool operator>=(Niche a, Niche b) { return !(a < b); }
};
// A position is an ordinal from 0: -1 is free.
using OptPos = Niche<int, int, -1>;
// For tables whose values are proven never to be Int.MIN.
using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<std::string> s;
std::vector<cpl::OptInt> ch0;
std::vector<cpl::OptInt> ch1;
std::vector<ll> ans;
ll cur = 0;

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
    std::optional<ll> t1;
    ll p1 = 0;
    n = cpl::read_int();
    s.assign(n, std::string());
    for (auto& t2 : s) t2 = cpl::read_str();
    ch0 = std::vector<cpl::OptInt>(n);
    ch1 = std::vector<cpl::OptInt>(n);
    for (ll i = 0; i < n; i++) {
        for (ll j = i - 1; j >= 0; j--) {
            if ((cpl::u8)s[j][i] == '1') {
                ch0[i] = cpl::OptInt(j);
            }
        }
        for (ll j_2 = i + 1; j_2 < n; j_2++) {
            if ((cpl::u8)s[j_2][i] == '1') {
                ch1[i] = cpl::OptInt(j_2);
            }
        }
    }
    ans = std::vector<ll>(n, 0);
    cur = 0;
    for (ll v = 1, v_end = n + 1; v < v_end; v++) {
        ans[cur] = v;
        std::optional<ll> pa0 = std::optional<ll>();
        std::optional<ll> pa1 = std::optional<ll>();
        for (ll j_3 = cur - 1; j_3 >= 0; j_3--) {
            if ((cpl::u8)s[cur][j_3] == '1') {
                pa0 = std::optional<ll>(j_3);
            }
        }
        for (ll j_4 = cur + 1; j_4 < n; j_4++) {
            if ((cpl::u8)s[cur][j_4] == '1') {
                pa1 = std::optional<ll>(j_4);
            }
        }
        std::optional<ll> t0 = pa0;
        if (t0.has_value()) {
            ll p0 = *t0;
            if (std::optional<ll>(ch1[p0]) == std::optional<ll>(cur)) {
                cur = p0;
            } else {
                t1 = pa1;
                if (t1.has_value()) {
                    p1 = *t1;
                    cur = p1;
                }
            }
        } else {
            t1 = pa1;
            if (t1.has_value()) {
                p1 = *t1;
                cur = p1;
            }
        }
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
