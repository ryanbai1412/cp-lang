// Matched C++17 reference for bench/math/matpow.cp (same flat row-major ikj Mat<uint32_t Mod<998244353>> ** 10^18).
#include <cstdint>
#include <iostream>
#include <vector>
using ll = long long;

static constexpr uint32_t P = 998244353;

struct Mat {
    int n;
    std::vector<uint32_t> data;
    Mat(int n_, uint32_t fill = 0) : n(n_), data(size_t(n_) * size_t(n_), fill) {}
    static Mat identity(int n) {
        Mat m(n, 0);
        for (int i = 0; i < n; ++i) m.data[i * n + i] = 1;
        return m;
    }
};

static Mat mul(const Mat& a, const Mat& b) {
    int n = a.n;
    Mat out(n, 0);
    for (int i = 0; i < n; ++i) {
        int ai = i * n, oi = i * n;
        for (int k = 0; k < n; ++k) {
            uint64_t f = a.data[ai + k];
            int bk = k * n;
            for (int j = 0; j < n; ++j) {
                uint32_t prod = uint32_t(f * b.data[bk + j] % P);
                uint32_t s = out.data[oi + j] + prod;
                if (s >= P) s -= P;
                out.data[oi + j] = s;
            }
        }
    }
    return out;
}

static Mat mpow(Mat base, ll e) {
    Mat acc = Mat::identity(base.n);
    while (e > 0) {
        if (e & 1) acc = mul(acc, base);
        e >>= 1;
        if (e > 0) base = mul(base, base);
    }
    return acc;
}

int main() {
    const int n = 80;
    Mat m(n, 0);
    ll s = 42;
    for (int i = 0; i < n * n; ++i) {
        s = (s * 1103515245LL + 12345LL) & 2147483647LL;
        m.data[i] = uint32_t(s % P);
    }
    Mat r = mpow(m, 1000000000000000000LL);
    uint64_t acc = 0;
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            acc = (acc + uint64_t(r.data[i * n + j]) * uint64_t(i * n + j + 1)) % P;
        }
    }
    std::cout << acc << '\n';
}
