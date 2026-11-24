#include <bits/stdc++.h>
int main() {
    int m;
    if (scanf("%d", &m) != 1) return 1;
    std::vector<std::pair<int, int>> given(m);
    for (auto& [s, e] : given) if (scanf("%d %d", &s, &e) != 2) return 1;
    std::vector<std::tuple<long long, bool, long long>> events;
    events.reserve(2 * m);
    for (int i = 0; i < m; i++) {
        events.emplace_back(-given[i].first, false, i);
        events.emplace_back(-given[i].second, true, i);
    }
    std::sort(events.begin(), events.end());
    long long h = 0;
    for (auto& [t, arrival, i] : events) h = ((h * 31 + t + (arrival ? 7 : 0) + i) % 1000000007 + 1000000007) % 1000000007;
    printf("%lld\n", h);
}
