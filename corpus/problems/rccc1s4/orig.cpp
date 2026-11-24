// Reference for Riolku's Mock CCC S4 (no accepted C++ on DMOJ; written for the corpus).
// x_i pours into bucket i; bucket i holds a_i x_i + b_{i-1} x_{i-1}. No x_i past
// max(ceil(c_i / a_i), ceil(c_{i+1} / b_i)) helps, so dp over (i, x_i) with a suffix minimum
// over x_{i-1} is O(N * 5000).
#include <bits/stdc++.h>
using namespace std;

int main() {
    int n;
    scanf("%d", &n);
    vector<long long> c(n + 1), a(n + 1), b(n + 1);
    for (int i = 1; i <= n; i++) scanf("%lld", &c[i]);
    for (int i = 1; i <= n; i++) scanf("%lld %lld", &a[i], &b[i]);
    auto cap = [&](int i) {
        long long m = (c[i] + a[i] - 1) / a[i];
        if (i < n) m = max(m, (c[i + 1] + b[i] - 1) / b[i]);
        return (int)m;
    };
    const long long INF = LLONG_MAX / 4;
    // prev[y]: least moves for buckets 1..i-1 filled with x_{i-1} = y; prev has one entry (0) at i = 1.
    vector<long long> prev(1, 0);
    long long bprev = 0;
    for (int i = 1; i <= n; i++) {
        int m = cap(i);
        vector<long long> suf(prev.size() + 1, INF);
        for (int y = (int)prev.size() - 1; y >= 0; y--) suf[y] = min(suf[y + 1], prev[y]);
        vector<long long> cur(m + 1, INF);
        for (int x = 0; x <= m; x++) {
            long long need = c[i] - a[i] * x, y = 0;
            if (need > 0) {
                if (bprev == 0) continue;
                y = (need + bprev - 1) / bprev;
            }
            if (y < (long long)prev.size() && suf[y] < INF) cur[x] = suf[y] + x;
        }
        prev = cur;
        bprev = b[i];
    }
    printf("%lld\n", *min_element(prev.begin(), prev.end()));
}
