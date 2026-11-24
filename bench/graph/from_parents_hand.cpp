// The matched C++: parents come before their children (p < i), so depth is one pass forward and
// subtree size one pass back, as cf2138c1's original does.
#include <cstdio>
#include <vector>
int main() {
    int t;
    if (scanf("%d", &t) != 1) return 1;
    std::vector<int> p, depth, sz;
    while (t--) {
        int n;
        if (scanf("%d", &n) != 1) return 1;
        p.assign(n + 1, 0);
        depth.assign(n + 1, 0);
        sz.assign(n + 1, 1);
        for (int i = 2; i <= n; i++) if (scanf("%d", &p[i]) != 1) return 1;
        for (int i = 2; i <= n; i++) depth[i] = depth[p[i]] + 1;
        for (int i = n; i >= 2; i--) sz[p[i]] += sz[i];
        long long s = 0;
        for (int i = 1; i <= n; i++) s += 3LL * depth[i] + sz[i];
        printf("%lld\n", s);
    }
}
