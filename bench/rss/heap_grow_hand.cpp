#include <algorithm>
#include <cstdio>
#include <utility>
#include <vector>
int main() {
    long long n;
    if (scanf("%lld", &n) != 1) return 1;
    std::vector<std::pair<long long, long long>> es;
    for (long long i = 0; i < 1000; i++) es.push_back({i * 7919 % 1000, i});
    std::stable_sort(es.begin(), es.end(), [](auto& x, auto& y) { return x.first < y.first; });
    std::vector<long long> a, b;
    for (long long i = 0; i < n; i++) {
        a.push_back(i + es[i % 1000].second);
        b.push_back(i * 2);
    }
    printf("%lld\n", a[n - 1] + b[n / 2] + (long long)a.size());
}
