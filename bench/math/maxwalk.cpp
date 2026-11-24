// Matched C++ for maxwalk.cp: the AC's flat max-plus product (corpus/submissions/3890258.cpp),
// row-major with a -2^62 sentinel, the powers squared up front.
#include <algorithm>
#include <cstdio>
#include <vector>
using ll = long long;
static const ll NEG = -(1LL << 62);
static const int n = 250;
using M = std::vector<ll>;
static M mul(const M& a, const M& b, int r) {
    M out(size_t(r) * n, NEG);
    for (int i = 0; i < r; ++i)
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
    ll s = 42;
    M g(size_t(n) * n, NEG);
    for (int i = 0; i < n; ++i)
        for (int t = 0; t < 3; ++t) {
            s = (s * 1103515245 + 12345) & 2147483647;
            g[i * n + s % 250] = s % 1000;
        }
    std::vector<M> pows{g};
    for (int i = 0; i < 30; ++i) pows.push_back(mul(pows.back(), pows.back(), n));
    M row(n, NEG);
    row[0] = 0;
    ll acc = 0;
    for (int t = 0; t < 200; ++t) {
        s = (s * 1103515245 + 12345) & 2147483647;
        ll e = s % 1000000 + 1;
        for (int b = 0; b < 30; ++b)
            if ((e >> b) & 1) row = mul(row, pows[b], 1);
        ll v = row[t % 250] == NEG ? 0 : row[t % 250];
        acc = (acc + v % 1000003) % 1000000007;
    }
    printf("%lld\n", acc);
}
