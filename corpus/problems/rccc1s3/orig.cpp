// Reference for Riolku's Mock CCC S3 (no accepted C++ on DMOJ; written for the corpus).
// Each connected component of the swap graph can realise any permutation of its positions, so
// the least result sorts every component's values into its sorted positions. To emit swaps,
// take a BFS spanning tree per component and repeatedly fix a leaf: walk its target value to
// it along the tree path, then drop the leaf. A component of s positions costs < s^2 swaps.
#include <bits/stdc++.h>
using namespace std;

int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<int> p(n + 1), pos(n + 1);
    for (int i = 1; i <= n; i++) scanf("%d", &p[i]), pos[p[i]] = i;
    vector<vector<int>> g(n + 1);
    for (int i = 0; i < m; i++) {
        int a, b;
        scanf("%d %d", &a, &b);
        g[a].push_back(b), g[b].push_back(a);
    }
    vector<int> comp(n + 1, 0), par(n + 1, 0), order;
    vector<pair<int, int>> swaps;
    auto do_swap = [&](int a, int b) {
        swap(p[a], p[b]);
        pos[p[a]] = a, pos[p[b]] = b;
        swaps.push_back({a, b});
    };
    for (int s = 1; s <= n; s++) {
        if (comp[s]) continue;
        vector<int> nodes{s};
        comp[s] = s;
        for (size_t k = 0; k < nodes.size(); k++)
            for (int v : g[nodes[k]])
                if (!comp[v]) comp[v] = s, par[v] = nodes[k], nodes.push_back(v);
        vector<int> sp(nodes), sv;
        sort(sp.begin(), sp.end());
        for (int v : nodes) sv.push_back(p[v]);
        sort(sv.begin(), sv.end());
        map<int, int> want;
        for (size_t k = 0; k < sp.size(); k++) want[sp[k]] = sv[k];
        // Tree adjacency on the BFS tree; leaves are fixed in reverse BFS order.
        vector<char> alive(n + 1, 0);
        for (int v : nodes) alive[v] = 1;
        vector<vector<int>> t(n + 1);
        for (int v : nodes)
            if (v != s) t[v].push_back(par[v]), t[par[v]].push_back(v);
        for (int k = (int)nodes.size() - 1; k >= 0; k--) {
            int leaf = nodes[k];
            int src = pos[want[leaf]];
            if (src != leaf) {
                // Path src -> leaf in the alive tree.
                map<int, int> from;
                from[src] = 0;
                deque<int> q{src};
                while (!q.empty()) {
                    int u = q.front();
                    q.pop_front();
                    if (u == leaf) break;
                    for (int v : t[u])
                        if (alive[v] && !from.count(v)) from[v] = u, q.push_back(v);
                }
                vector<int> path;
                for (int u = leaf; u; u = from[u]) path.push_back(u);
                reverse(path.begin(), path.end());
                for (size_t j = 0; j + 1 < path.size(); j++) do_swap(path[j], path[j + 1]);
            }
            alive[leaf] = 0;
        }
    }
    printf("%d\n", (int)swaps.size());
    for (int i = 1; i <= n; i++) printf("%d%c", p[i], i == n ? '\n' : ' ');
    for (auto [a, b] : swaps) printf("%d %d\n", a, b);
}
