// Matched C++17 reference for bench/math/crt.cp (same iterative ext_gcd + CRT algorithm).
#include <cstdint>
#include <iostream>
#include <optional>
#include <utility>
using ll = long long;

static inline ll emod(ll a, ll m) {
    ll r = a % m;
    return r < 0 ? r + m : r;
}

static inline ll ediv(ll a, ll b) {
    ll q = a / b, r = a % b;
    if (r < 0) q += (b > 0 ? -1 : 1);
    return q;
}

static inline ll mul_mod(ll a, ll b, ll m) {
    if (-3037000499LL <= a && a <= 3037000499LL && -3037000499LL <= b && b <= 3037000499LL) {
        return emod(a * b, m);
    }
    return ll((__int128(a) * __int128(b)) % __int128(m) + __int128(m)) % m;
}

struct ExtGcd { ll g, x, y; };

static inline ExtGcd ext_gcd(ll a, ll b) {
    ll oa = a, ob = b, ox = 1, x = 0, oy = 0, y = 1;
    while (ob != 0) {
        ll q = ediv(oa, ob);
        ll na = ob, nb = oa - q * ob;
        oa = na; ob = nb;
        ll nx = x, ny = ox - q * x;
        ox = nx; x = ny;
        ll my = y, mny = oy - q * y;
        oy = my; y = mny;
    }
    if (oa < 0) { oa = -oa; ox = -ox; oy = -oy; }
    return {oa, ox, oy};
}

static inline std::optional<std::pair<ll, ll>> crt(ll r1, ll m1, ll r2, ll m2) {
    auto e = ext_gcd(m1, m2);
    if (emod(r2 - r1, e.g) != 0) return std::nullopt;
    ll m = m1 / e.g * m2;
    ll step = m2 / e.g;
    ll k = mul_mod(emod(ediv(r2 - r1, e.g), step), emod(e.x, step), step);
    return std::make_pair(emod(r1 + m1 * k, m), m);
}

int main() {
    const int n = 1000000;
    ll s = 987654321, ok = 0, acc = 0;
    for (int i = 0; i < n; ++i) {
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        ll m1 = s % 100000 + 2;
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        ll m2 = s % 100000 + 2;
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        ll r1 = s % m1;
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        ll r2 = s % m2;
        if (auto res = crt(r1, m1, r2, m2)) {
            ok += 1;
            acc = (acc + res->first + res->second) % 1000000007LL;
        }
    }
    std::cout << ok << ' ' << acc << '\n';
}
