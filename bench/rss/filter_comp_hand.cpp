#include <bits/stdc++.h>
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 1;
    std::vector<long long> a(n);
    for (auto& x : a) if (scanf("%lld", &x) != 1) return 1;
    std::vector<std::pair<long long, long long>> even, few;
    for (long long x : a) if (x % 2 == 0) even.push_back({x, 2 * x});
    for (long long x : a) if (x % 1000 == 0) few.push_back({x, 2 * x});
    printf("%zu %zu %lld\n", even.size(), few.size(), even.back().second + few[0].second);
}
