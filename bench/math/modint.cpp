// Matched C++17 reference for bench/math/modint.cp (same uint32_t Mod<998244353> layout & ext_gcd inv).
#include <cstdint>
#include <iostream>
using ll = long long;

struct Mod {
    static constexpr ll P = 998244353;
    uint32_t v;
    Mod(ll x = 0) {
        ll r = x % P;
        if (r < 0) r += P;
        v = uint32_t(r);
    }
    Mod operator+(Mod o) const {
        ll s = ll(v) + ll(o.v);
        if (s >= P) s -= P;
        return Mod::raw(uint32_t(s));
    }
    Mod operator-(Mod o) const {
        ll s = ll(v) - ll(o.v);
        if (s < 0) s += P;
        return Mod::raw(uint32_t(s));
    }
    Mod operator*(Mod o) const {
        return Mod::raw(uint32_t(ll(v) * ll(o.v) % P));
    }
    Mod inv() const {
        ll a = v, b = P, x = 1, y = 0;
        while (b != 0) {
            ll q = a / b;
            ll na = b, nb = a - q * b;
            a = na; b = nb;
            ll nx = y, ny = x - q * y;
            x = nx; y = ny;
        }
        return Mod(x);
    }
    Mod operator/(Mod o) const { return *this * o.inv(); }
    Mod pow(ll e) const {
        Mod base = *this, acc = Mod::raw(1);
        while (e > 0) {
            if (e & 1) acc = acc * base;
            base = base * base;
            e >>= 1;
        }
        return acc;
    }
    static Mod raw(uint32_t x) {
        Mod m;
        m.v = x;
        return m;
    }
};

int main() {
    const ll n = 10000000;
    Mod a(1234567), b(89012345), acc(0);
    for (ll i = 1; i <= n; ++i) {
        Mod c = a * b + Mod(i);
        a = b - c;
        b = c + a;
        acc = acc + b;
    }
    Mod inv_acc(0), x(314159265);
    for (ll i = 1; i <= 200000; ++i) {
        inv_acc = inv_acc + x.pow(17) / Mod(i);
        x = x * Mod::raw(3) + Mod::raw(1);
    }
    std::cout << (acc + inv_acc).v << '\n';
}
