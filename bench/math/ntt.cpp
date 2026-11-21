// Matched C++17 reference for bench/math/ntt.cp (same Montgomery radix-2 DIF/DIT NTT, uint32_t Mod<998244353>).
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

static constexpr uint32_t P = 998244353;
static constexpr uint32_t G = 3;

static uint32_t pow_mod(uint32_t a, uint32_t e) {
    uint64_t r = 1, b = a;
    while (e) {
        if (e & 1) r = r * b % P;
        b = b * b % P;
        e >>= 1;
    }
    return uint32_t(r);
}

static uint32_t mont_inv(uint32_t p) {
    uint32_t x = p;
    for (int i = 0; i < 4; ++i) x *= 2u - p * x;
    return uint32_t(0u - x);
}

static inline uint32_t mred(uint64_t x, uint32_t pinv) {
    uint32_t t = uint32_t(x) * pinv;
    uint32_t r = uint32_t((x + uint64_t(t) * P) >> 32);
    return r >= P ? r - P : r;
}

static void ntt_dif(std::vector<uint32_t>& a, uint32_t pinv, uint32_t r1, uint32_t r2) {
    int n = int(a.size());
    std::vector<uint32_t> roots(n >> 1);
    for (int i = 0; i < n; ++i) a[i] = mred(uint64_t(a[i]) * r2, pinv);
    for (int len = n; len >= 2; len >>= 1) {
        int half = len >> 1;
        uint32_t wlen = mred(uint64_t(pow_mod(G, (P - 1) / len)) * r2, pinv);
        roots[0] = r1;
        for (int k = 1; k < half; ++k) roots[k] = mred(uint64_t(roots[k - 1]) * wlen, pinv);
        for (int i = 0; i < n; i += len) {
            for (int k = 0; k < half; ++k) {
                uint32_t u = a[i + k], v = a[i + k + half];
                uint32_t x = u + v;
                if (x >= P) x -= P;
                uint32_t y = u >= v ? u - v : u + P - v;
                a[i + k] = x;
                a[i + k + half] = mred(uint64_t(y) * roots[k], pinv);
            }
        }
    }
}

static void ntt_dit(std::vector<uint32_t>& a, uint32_t pinv, uint32_t r1, uint32_t r2) {
    int n = int(a.size());
    std::vector<uint32_t> roots(n >> 1);
    for (int len = 2; len <= n; len <<= 1) {
        int half = len >> 1;
        uint32_t wlen = mred(uint64_t(pow_mod(pow_mod(G, (P - 1) / len), P - 2)) * r2, pinv);
        roots[0] = r1;
        for (int k = 1; k < half; ++k) roots[k] = mred(uint64_t(roots[k - 1]) * wlen, pinv);
        for (int i = 0; i < n; i += len) {
            for (int k = 0; k < half; ++k) {
                uint32_t u = a[i + k];
                uint32_t v = mred(uint64_t(a[i + k + half]) * roots[k], pinv);
                uint32_t x = u + v;
                if (x >= P) x -= P;
                uint32_t y = u >= v ? u - v : u + P - v;
                a[i + k] = x;
                a[i + k + half] = y;
            }
        }
    }
    uint32_t inv_n = pow_mod(uint32_t(n), P - 2);
    for (int i = 0; i < n; ++i) a[i] = mred(uint64_t(a[i]) * inv_n, pinv);
}

int main() {
    const int n = 262144;
    std::vector<uint32_t> a(n), b(n);
    ll s = 123456789;
    for (int i = 0; i < n; ++i) {
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        a[i] = uint32_t(s % P);
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        b[i] = uint32_t(s % P);
    }
    int n_out = 2 * n - 1;
    int sz = 1;
    while (sz < n_out) sz <<= 1;
    a.resize(sz, 0);
    b.resize(sz, 0);
    uint32_t pinv = mont_inv(P);
    uint32_t r1 = uint32_t((1ULL << 32) % P);
    uint32_t r2 = uint32_t(uint64_t(r1) * r1 % P);
    ntt_dif(a, pinv, r1, r2);
    ntt_dif(b, pinv, r1, r2);
    for (int i = 0; i < sz; ++i) a[i] = mred(uint64_t(a[i]) * b[i], pinv);
    ntt_dit(a, pinv, r1, r2);
    a.resize(n_out);
    uint64_t acc = 0;
    for (int i = 0; i < n_out; ++i) acc = (acc + uint64_t(a[i]) * uint64_t((i + 1) % P)) % P;
    std::cout << acc << '\n';
}
