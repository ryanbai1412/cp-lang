#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::pair<long long, long long>> xs(n);
    for (int i = 0; i < n; i++) xs[i] = {(long long)i * 7919 % 1000003, i};
    std::vector<std::pair<long long, long long>> s = xs;
    std::stable_sort(s.begin(), s.end(), [](const auto& p, const auto& q) { return p.first % 1000 < q.first % 1000; });
    long long h = 0;
    for (auto& [a, b] : s) h = (h * 31 + a + b) % 1000000007;
    printf("%lld\n", h);
}
