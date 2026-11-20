// cf2201a2.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2201 A2: start from total subarray lengths and subtract (p + 1) * (N - i)
//     // whenever element i unlinks against its live predecessor p with value a - 1.
//     @!multitest
//     let n: Int = read()
//     let a: [0..n]Int = read()
//     let vals = sorted([(-a[i], i) for i in 0..n])
//
//     var prv: [0..n]?Int = none
//     var nxt: [0..n]?Int = none
//     for i in 0..n:
//         if i > 0: prv[i] = i - 1
//         if i + 1 < n: nxt[i] = i + 1
//
//     var ans = n * (n + 1) * (n + 2) / 6
//     for (neg_v, i) in vals:
//         let v = -neg_v
//         if let p = prv[i]:
//             if a[p] == v - 1:
//                 ans -= (p + 1) * (n - i)
//                 let nx = nxt[i]
//                 nxt[p] = nx
//                 if let q = nx: prv[q] = p
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
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    return a / b + ((a % b) >> 63);
}
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
template <class T> std::vector<T> sorted(std::vector<T> v) { std::sort(v.begin(), v.end()); return v; }
template <class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<ll> a;
std::vector<std::tuple<ll, ll>> vals;
std::vector<cpl::OptInt> prv;
std::vector<cpl::OptInt> nxt;
ll ans = 0;


bool setup() {
    return true;
}

void solve() {
    n = cpl::read_int();
    a.assign(n, ll());
    for (auto& t4 : a) t4 = cpl::read_int();
    vals = cpl::sorted([&]() -> std::vector<std::tuple<ll, ll>> {
        std::vector<std::tuple<ll, ll>> t0 = std::vector<std::tuple<ll, ll>>();
        for (ll i = 0; i < n; i++) {
            t0.push_back(std::tuple<ll, ll>{-a[i], i});
        }
        return t0;
    }());
    prv = std::vector<cpl::OptInt>(n);
    nxt = std::vector<cpl::OptInt>(n);
    for (ll i_2 = 0; i_2 < n; i_2++) {
        if (i_2 > 0) {
            prv[i_2] = cpl::OptInt(i_2 - 1);
        }
        if (i_2 + 1 < n) {
            nxt[i_2] = cpl::OptInt(i_2 + 1);
        }
    }
    ans = cpl::divp(n * (n + 1) * (n + 2), 6);
    for (const std::tuple<ll, ll>& t1 : vals) {
        ll neg_v = std::get<0>(t1);
        ll i_3 = std::get<1>(t1);
        ll v = -neg_v;
        std::optional<ll> t2 = std::optional<ll>(prv[i_3]);
        if (t2.has_value()) {
            ll p = *t2;
            if (a[p] == v - 1) {
                ans -= (p + 1) * (n - i_3);
                std::optional<ll> nx = std::optional<ll>(nxt[i_3]);
                nxt[p] = cpl::OptInt(nx);
                std::optional<ll> t3 = nx;
                if (t3.has_value()) {
                    ll q = *t3;
                    prv[q] = cpl::OptInt(p);
                }
            }
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
