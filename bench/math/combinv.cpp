// Matched C++17 reference for bench/math/combinv.cp: a hand-hoisted table of 1 / k.
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

static constexpr uint32_t P = 1000000007;

static uint32_t pow_mod(uint64_t b, uint64_t e) {
    uint64_t r = 1;
    for (; e; e >>= 1, b = b * b % P)
        if (e & 1) r = r * b % P;
    return uint32_t(r);
}

int main() {
    const int lim = 5000;
    std::vector<uint32_t> f(lim + 1), g(lim + 1), inv(lim + 1);
    f[0] = 1;
    for (int i = 1; i <= lim; ++i) f[i] = uint32_t(uint64_t(f[i - 1]) * uint64_t(i) % P);
    g[lim] = pow_mod(f[lim], P - 2);
    for (int i = lim; i >= 1; --i) g[i - 1] = uint32_t(uint64_t(g[i]) * uint64_t(i) % P);
    for (int i = 1; i <= lim; ++i) inv[i] = uint32_t(uint64_t(g[i]) * f[i - 1] % P);

    ll s = 246813579;
    uint32_t acc = 0;
    for (int it = 0; it < 25000000; ++it) {
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        uint32_t v = uint32_t(uint64_t(s % P) * inv[s % 5000 + 1] % P);
        acc += v;
        if (acc >= P) acc -= P;
    }
    std::cout << acc << '\n';
}
