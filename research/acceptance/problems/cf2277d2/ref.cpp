// f(n) = min(f(n - 2^k + 2) + k + 1, 2^(k+1) - n + k) for 2^k <= n < 2^(k+1), n >= 4
// (editorial). f < B everywhere, so the second option only wins on the last B values
// of each [2^k, 2^(k+1)); elsewhere f(n) = f(n - 2^k + 2) + k + 1, and the prefix sum
// g(n) reduces to g at an argument below 2^k plus a precomputed tail sum.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef unsigned long long ull;
const ll MOD = 998244353, B = 2048;
const int K = 60;
ll G[K + 1];
vector<ll> tailPre[K];
ll tailStart[K];

ll f(ll n) {
    if (n < 4) return (ll[]){0, 1, 3, 2}[n];
    int k = 63 - __builtin_clzll(n);
    return min(f(n - (1LL << k) + 2) + k + 1, (1LL << (k + 1)) - n + k);
}

ll g(ll n) {
    if (n < 0) return 0;
    if (n < 4) return (ll[]){0, 1, 4, 6}[n];
    int k = 63 - __builtin_clzll(n);
    ll lo = 1LL << k, w = (1LL << (k + 1)) - B, res = G[k];
    if (w >= lo) {
        ll e = min(n, w);
        res += g(e - lo + 2) - 1 + (e - lo + 1) % MOD * (k + 1);
    }
    if (n >= tailStart[k]) res += tailPre[k][n - tailStart[k]];
    return res % MOD;
}

int main() {
    for (int k = 2; k < K; k++) {
        tailStart[k] = max(1LL << k, (1LL << (k + 1)) - B + 1);
        ll s = 0;
        for (ll i = tailStart[k]; i < (1LL << (k + 1)); i++) tailPre[k].push_back(s = (s + f(i)) % MOD);
    }
    G[2] = 6;
    for (int k = 2; k < K; k++) G[k + 1] = g((1LL << (k + 1)) - 1);
    int t;
    if (scanf("%d", &t) != 1) return 0;
    while (t--) {
        ll l, r;
        if (scanf("%lld %lld", &l, &r) != 2) return 0;
        printf("%lld\n", ((g(r) - g(l - 1)) % MOD + MOD) % MOD);
    }
}
