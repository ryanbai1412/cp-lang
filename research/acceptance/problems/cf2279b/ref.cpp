#include <bits/stdc++.h>
using namespace std;
const long long MOD = 998244353;
int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        int n;
        scanf("%d", &n);
        int m = 3 * n + 3;
        vector<char> in(m + 1, 0);
        for (int i = 0; i < n; i++) { int x; scanf("%d", &x); in[x] = 1; }
        // chains with >= 2 gaps, gaps in {1,2}, no "2 2", no "1 1 1"
        // g2: last gap 2; o1: last gap 1 (one 1); o2: last two gaps 1 1; e: chain of one element
        vector<array<long long, 3>> f(m + 1, {0, 0, 0});
        long long big = 0;
        for (int v = 1; v <= m; v++) {
            if (!in[v]) continue;
            long long g2 = 0, o1 = 0, o2 = 0;
            if (v >= 2 && in[v - 2]) { g2 = (1 + f[v - 2][1] + f[v - 2][2]) % MOD; big += f[v - 2][1] + f[v - 2][2]; }
            if (v >= 1 && in[v - 1]) { o1 = (1 + f[v - 1][0]) % MOD; o2 = f[v - 1][1]; big += f[v - 1][0] + f[v - 1][1]; }
            f[v] = {g2, o1, o2};
            big %= MOD;
        }
        long long ln = n;
        printf("%lld\n", (big + ln + ln * (ln - 1) / 2) % MOD);
    }
}
