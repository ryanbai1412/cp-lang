#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<std::pair<long long, long long>> ps(n);
    for (auto& [a, b] : ps) if (scanf("%lld %lld", &a, &b) != 2) return 1;
    std::stable_sort(ps.begin(), ps.end());
    printf("%lld\n", ps[0].first + ps[n - 1].second);
}
