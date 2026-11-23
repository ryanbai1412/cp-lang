#include <bits/stdc++.h>
using namespace std;
int main() {
    int t; scanf("%d", &t);
    while (t--) {
        int n, k; scanf("%d %d", &n, &k);
        vector<int> p(n + 1, 0);
        for (int i = 1; i <= n; i++) { int x; scanf("%d", &x); p[i] = (int)((p[i - 1] + (long long)x) % k); }
        vector<int> cnt(k, 0);
        long long ans = 0;
        for (int j = k + 1; j <= n; j++) { cnt[p[j - k - 1]]++; ans += cnt[p[j]]; }
        printf("%lld\n", ans);
    }
}
