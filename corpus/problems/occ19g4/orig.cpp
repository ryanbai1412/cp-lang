// Brute-force reference: every pair of geese, O(N^2). Goose p honks at r when
// dist(p, r) < dist(p, mate of p); a pair that honks both ways fights once.
#include <cstdio>
#include <cstdlib>
#pragma GCC optimize("O3")
#pragma GCC target("avx2")

static int x[120000], y[120000], r[120000], fights[120000];

int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    int m = 2 * n;
    for (int i = 0; i < m; i++) scanf("%d %d", &x[i], &y[i]);
    for (int i = 0; i < m; i++) {
        int j = i ^ 1;
        r[i] = abs(x[i] - x[j]) + abs(y[i] - y[j]) - 1;
    }
    for (int i = 0; i < m; i++) {
        int xi = x[i], yi = y[i], ri = r[i], own = 0;
        for (int j = i + 1; j < m; j++) {
            int d = abs(xi - x[j]) + abs(yi - y[j]);
            int c = (d <= ri) & (d <= r[j]);
            own += c;
            fights[j] += c;
        }
        fights[i] += own;
    }
    for (int i = 0; i < m; i++) printf("%d\n", fights[i]);
}
