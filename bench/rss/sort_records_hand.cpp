#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::pair<long long, long long>> xs(n);
    for (auto& [a, b] : xs) if (scanf("%lld %lld", &a, &b) != 2) return 1;
    std::stable_sort(xs.begin(), xs.end(), [](auto& p, auto& q) { return p.first < q.first; });
    printf("%lld\n", xs[0].second + xs[n - 1].second);
}
