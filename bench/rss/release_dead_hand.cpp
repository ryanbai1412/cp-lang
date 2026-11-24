#include <cstdio>
#include <vector>

int main() {
    int n, m;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    std::vector<std::pair<int, int>> arcs(m);
    for (auto& [u, v] : arcs)
        if (scanf("%d %d", &u, &v) != 2) return 1;
    std::vector<std::vector<int>> g(n);
    for (auto [u, v] : arcs) g[u].push_back(v);
    std::vector<std::pair<int, int>>().swap(arcs);
    std::vector<long long> big(4LL * m);
    for (long long i = 0; i < 4LL * m; i++) big[i] = (long long)g[i % n].size() + i;
    printf("%lld\n", big[4LL * m - 1] + big[0]);
}
