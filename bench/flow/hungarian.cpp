#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
int main() {
    int n = 1500, m = 1500;
    vector<vector<long long>> w(n, vector<long long>(m));
    for (auto& r : w) for (auto& x : r) x = nxt(1000000000);
    vector<long long> u(n + 1), v(m + 1), minv(m + 1); vector<int> p(m + 1), way(m + 1); vector<char> used(m + 1);
    for (int i = 1; i <= n; i++) {
        p[0] = i; int j0 = 0;
        fill(minv.begin(), minv.end(), LLONG_MAX); fill(used.begin(), used.end(), 0);
        do { used[j0] = 1; int i0 = p[j0], j1 = 0; long long delta = LLONG_MAX;
            for (int j = 1; j <= m; j++) if (!used[j]) { long long cur = w[i0 - 1][j - 1] - u[i0] - v[j];
                if (cur < minv[j]) { minv[j] = cur; way[j] = j0; } if (minv[j] < delta) { delta = minv[j]; j1 = j; } }
            for (int j = 0; j <= m; j++) if (used[j]) { u[p[j]] += delta; v[j] -= delta; } else minv[j] -= delta;
            j0 = j1; } while (p[j0] != 0);
        do { int j1 = way[j0]; p[j0] = p[j1]; j0 = j1; } while (j0);
    }
    long long tot = 0; for (int j = 1; j <= m; j++) if (p[j]) tot += w[p[j] - 1][j - 1];
    printf("%lld\n", tot);
}
