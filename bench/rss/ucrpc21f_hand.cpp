#include <bits/stdc++.h>
// Dijkstra over the n x n grid: one 8-byte distance per cell and a heap of (d, x, y).
int main() {
    int n, t;
    if (scanf("%d %d", &n, &t) != 2) return 1;
    std::vector<std::string> g(n);
    static char buf[1 << 11];
    for (auto& row : g) {
        if (scanf("%s", buf) != 1) return 1;
        row = buf;
    }
    std::vector<long long> dist((size_t)n * n, LLONG_MAX);
    std::priority_queue<std::tuple<long long, int, int>, std::vector<std::tuple<long long, int, int>>,
                        std::greater<>> pq;
    dist[0] = 0;
    pq.emplace(0, 0, 0);
    const int dx[] = {-1, 1, 0, 0}, dy[] = {0, 0, -1, 1};
    while (!pq.empty()) {
        auto [d, x, y] = pq.top();
        pq.pop();
        if (d != dist[(size_t)x * n + y]) continue;
        for (int k = 0; k < 4; k++) {
            int a = x + dx[k], b = y + dy[k];
            if (a < 0 || a >= n || b < 0 || b >= n) continue;
            long long nd = d + 1 + (g[a][b] == 'b' ? t : 0);
            if (nd < dist[(size_t)a * n + b]) {
                dist[(size_t)a * n + b] = nd;
                pq.emplace(nd, a, b);
            }
        }
    }
    printf("%lld\n", dist[(size_t)n * n - 1]);
}
