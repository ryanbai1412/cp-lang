// Matched C++ reference for bench/math/maxplus.cp: flat row-major ikj max-plus ** 10^6 with a -2^62 sentinel.
#include <algorithm>
#include <cstdio>
#include <vector>
using ll = long long;
static const ll NEG = -(1LL << 62);
static const int n = 200;
using M = std::vector<ll>;
static M mul(const M& a, const M& b) {
    M out(size_t(n) * n, NEG);
    for (int i = 0; i < n; ++i)
        for (int k = 0; k < n; ++k) {
            ll f = a[i * n + k];
            if (f == NEG) continue;
            const ll* bk = &b[k * n];
            ll* oi = &out[i * n];
            for (int j = 0; j < n; ++j) oi[j] = std::max(oi[j], f + bk[j]);
        }
    for (ll& v : out) if (v < NEG / 2) v = NEG;
    return out;
}
int main() {
    M m(size_t(n) * n, NEG);
    ll s = 42;
    for (int i = 0; i < n * n; ++i) {
        s = (s * 1103515245 + 12345) & 2147483647;
        if (s % 3 != 0) m[i] = s % 1000;
    }
    M acc(size_t(n) * n, NEG);
    for (int i = 0; i < n; ++i) acc[i * n + i] = 0;
    M base = m;
    for (ll k = 1000000; k > 0;) {
        if (k & 1) acc = mul(acc, base);
        k >>= 1;
        if (k > 0) base = mul(base, base);
    }
    ll sum = 0, gaps = 0;
    for (int i = 0; i < n; ++i)
        for (int j = 0; j < n; ++j) {
            ll v = acc[i * n + j];
            if (v == NEG) ++gaps;
            else sum = (sum + v % 1000003 * (i * n + j + 1)) % 1000000007;
        }
    printf("%lld %lld\n", sum, gaps);
}
