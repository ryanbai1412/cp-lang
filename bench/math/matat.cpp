// Matched C++ for matat.cp: a flat n*n array of residues mod 998244353.
#include <cstdio>
#include <vector>
using ll = long long;
const ll MOD = 998244353;
int main() {
    const int n = 2000;
    std::vector<ll> a((size_t)n * n, 0), m((size_t)n * n, 0);
    for (int i = 0; i < n; i++) a[(size_t)i * n + i] = 1;
    for (ll k = 0; k < 8; k++)
        for (int r = 0; r < n; r++)
            for (int c = 0; c < n; c++) {
                ll &x = m[(size_t)r * n + c];
                x = (x + (a[(size_t)r * n + c] + k) % MOD) % MOD;
            }
    ll s = 0;
    for (int r = 0; r < n; r++) s = (s + m[(size_t)r * n + r]) % MOD;
    printf("%lld\n", s);
}
