// Matched C++ for matwalk.cp: the AC's fixed 50x50 product, `(ret + a * b) % mod`.
#include <array>
#include <cstdio>
using ll = long long;
const ll mod = 1000000007;
const int n = 50;
using mat = std::array<std::array<ll, n>, n>;
mat mul(const mat& a, const mat& b) {
    mat r{};
    for (int i = 0; i < n; i++)
        for (int k = 0; k < n; k++)
            for (int j = 0; j < n; j++) r[i][j] = (r[i][j] + a[i][k] * b[k][j]) % mod;
    return r;
}
int main() {
    ll acc = 0;
    for (int t = 0; t < 20; t++) {
        ll s = 7 + t;
        mat m{};
        for (int i = 0; i < n * n; i++) {
            s = (s * 1103515245 + 12345) & 2147483647;
            m[i / n][i % n] = s >> 30 & 1;
        }
        mat r{};
        for (int i = 0; i < n; i++) r[i][i] = 1;
        for (ll e = 1000000000000000000LL; e > 0; e >>= 1) {
            if (e & 1) r = mul(r, m);
            if (e > 1) m = mul(m, m);
        }
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++) acc = (acc + r[i][j]) % mod;
    }
    printf("%lld\n", acc);
}
