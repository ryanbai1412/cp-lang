// An Animal Contest 5 P6: Larry starts at node 1 and may teleport from u to any v within tree
// distance D for a_v. The cost of entering v does not depend on u, so in Dijkstra order the first
// node to reach v gives its final cost. Centroid decomposition finds the unvisited nodes within D
// of u: for each centroid ancestor c of u, c's component is kept sorted by distance to c, and a
// pointer marks the prefix already reached; everything up to D - dist(c, u) past it is new.
#include <algorithm>
#include <cstdio>
#include <functional>
#include <queue>
#include <vector>

int main() {
    int n, d;
    std::scanf("%d %d", &n, &d);
    std::vector<long long> a(n);
    for (auto& x : a) std::scanf("%lld", &x);
    std::vector<std::vector<int>> adj(n);
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        std::scanf("%d %d", &u, &v);
        u--, v--;
        adj[u].push_back(v);
        adj[v].push_back(u);
    }
    // anc[v]: (centroid, distance) pairs; lists[c]: (distance, node) sorted
    std::vector<std::vector<std::pair<int, int>>> anc(n);
    std::vector<std::vector<std::pair<int, int>>> lists(n);
    std::vector<char> removed(n, 0);
    std::vector<int> sub(n), order, par(n), dep(n);
    std::vector<int> stack{0};
    std::vector<int> roots{0};
    while (!roots.empty()) {
        int r = roots.back();
        roots.pop_back();
        // sizes in r's component
        order.clear();
        par[r] = -1;
        stack.assign(1, r);
        while (!stack.empty()) {
            int u = stack.back();
            stack.pop_back();
            order.push_back(u);
            for (int v : adj[u])
                if (!removed[v] && v != par[u]) par[v] = u, stack.push_back(v);
        }
        for (int i = (int)order.size() - 1; i >= 0; i--) {
            int u = order[i];
            sub[u] = 1;
            for (int v : adj[u])
                if (!removed[v] && v != par[u]) sub[u] += sub[v];
        }
        int total = (int)order.size(), c = r;
        for (;;) {
            int next = -1;
            for (int v : adj[c])
                if (!removed[v] && v != par[c] && sub[v] * 2 > total) next = v;
            if (next < 0) break;
            c = next;
        }
        // distances from c over its component (BFS gives them sorted)
        auto& L = lists[c];
        par[c] = -1;
        dep[c] = 0;
        std::queue<int> bfs;
        bfs.push(c);
        while (!bfs.empty()) {
            int u = bfs.front();
            bfs.pop();
            L.push_back({dep[u], u});
            anc[u].push_back({c, dep[u]});
            for (int v : adj[u])
                if (!removed[v] && v != par[u]) par[v] = u, dep[v] = dep[u] + 1, bfs.push(v);
        }
        removed[c] = 1;
        for (int v : adj[c])
            if (!removed[v]) roots.push_back(v);
    }
    std::vector<size_t> ptr(n, 0);
    std::vector<long long> dist(n, -1);
    using Item = std::pair<long long, int>;
    std::priority_queue<Item, std::vector<Item>, std::greater<Item>> pq;
    dist[0] = 0;
    pq.push({0, 0});
    while (!pq.empty()) {
        auto [du, u] = pq.top();
        pq.pop();
        for (auto [c, dc] : anc[u]) {
            auto& L = lists[c];
            size_t& p = ptr[c];
            while (p < L.size() && L[p].first <= d - dc) {
                int v = L[p++].second;
                if (dist[v] < 0) dist[v] = du + a[v], pq.push({dist[v], v});
            }
        }
    }
    for (int i = 0; i < n; i++) std::printf("%lld%c", dist[i], i + 1 < n ? ' ' : '\n');
}
