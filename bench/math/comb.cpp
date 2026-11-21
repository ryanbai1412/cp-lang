// Matched C++17 reference for bench/math/comb.cp (same uint32_t factorial/inv-factorial table).
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

static constexpr uint32_t P = 998244353;

static uint32_t inv_mod(uint32_t v) {
    ll a = v, b = P, x = 1, y = 0;
    while (b != 0) {
        ll q = a / b;
        ll na = b, nb = a - q * b;
        a = na; b = nb;
        ll nx = y, ny = x - q * y;
        x = nx; y = ny;
    }
    x %= P;
    if (x < 0) x += P;
    return uint32_t(x);
}

int main() {
    const int lim = 2000000;
    std::vector<uint32_t> f(lim + 1), g(lim + 1);
    f[0] = 1;
    for (int i = 1; i <= lim; ++i) f[i] = uint32_t(uint64_t(f[i - 1]) * uint64_t(i) % P);
    g[lim] = inv_mod(f[lim]);
    for (int i = lim; i >= 1; --i) g[i - 1] = uint32_t(uint64_t(g[i]) * uint64_t(i) % P);

    ll s = 246813579;
    uint32_t acc = 0;
    for (int it = 0; it < 5000000; ++it) {
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        int n = int(s % (lim + 1));
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        int k = int(s % (n + 1));
        uint32_t v = uint32_t(uint64_t(f[n]) * g[k] % P * g[n - k] % P);
        acc += v;
        if (acc >= P) acc -= P;
    }
    std::cout << acc << '\n';
}
