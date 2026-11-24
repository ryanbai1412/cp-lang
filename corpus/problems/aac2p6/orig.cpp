// An Animal Contest 2 P6 (Koala Transport System). Written for the corpus: both
// submissions TLE. Answer = sum depth(x) - max over v of sum depth(lca(x, v)); the best v
// is a node of the virtual tree of the day's grounds, and a root path's gain sums, per
// virtual edge (p, u), (depth u - depth p) * (grounds under u).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, q;
    scanf("%d %d", &n, &q);
    vector<vector<int>> g(n + 1);
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        scanf("%d %d", &u, &v);
        g[u].push_back(v), g[v].push_back(u);
    }
    int LOG = 1;
    while ((1 << LOG) <= n) LOG++;
    vector<vector<int>> up(LOG, vector<int>(n + 1, 1));
    vector<int> dep(n + 1, 0), tin(n + 1), tout(n + 1);
    vector<pair<int, int>> st{{1, 0}};
    int timer = 0;
    tin[1] = timer++;
    while (!st.empty()) {
        auto &[u, i] = st.back();
        if (i < (int)g[u].size()) {
            int v = g[u][i++];
            if (v == up[0][u] && u != 1) continue;
            if (v == 1) continue;
            up[0][v] = u, dep[v] = dep[u] + 1, tin[v] = timer++;
            st.push_back({v, 0});
        } else {
            tout[u] = timer++;
            st.pop_back();
        }
    }
    for (int k = 1; k < LOG; k++)
        for (int v = 1; v <= n; v++) up[k][v] = up[k - 1][up[k - 1][v]];
    auto anc = [&](int a, int b) { return tin[a] <= tin[b] && tout[b] <= tout[a]; };
    auto lca = [&](int a, int b) {
        if (anc(a, b)) return a;
        if (anc(b, a)) return b;
        for (int k = LOG - 1; k >= 0; k--)
            if (!anc(up[k][a], b)) a = up[k][a];
        return up[0][a];
    };
    vector<long long> cnt(n + 1, 0), best(n + 1, 0);
    vector<int> par(n + 1, 0);
    while (q--) {
        int x;
        scanf("%d", &x);
        vector<int> xs(x);
        long long total = 0;
        for (auto &v : xs) scanf("%d", &v), total += dep[v];
        vector<int> nodes = xs;
        nodes.push_back(1);
        auto byTin = [&](int a, int b) { return tin[a] < tin[b]; };
        sort(nodes.begin(), nodes.end(), byTin);
        int m = nodes.size();
        for (int i = 0; i + 1 < m; i++) nodes.push_back(lca(nodes[i], nodes[i + 1]));
        sort(nodes.begin(), nodes.end(), byTin);
        nodes.erase(unique(nodes.begin(), nodes.end()), nodes.end());
        for (int v : nodes) cnt[v] = 0;
        for (int v : xs) cnt[v] = 1;
        vector<int> stk;
        for (int v : nodes) {
            while (!stk.empty() && !anc(stk.back(), v)) stk.pop_back();
            par[v] = stk.empty() ? 0 : stk.back();
            stk.push_back(v);
        }
        for (int i = nodes.size() - 1; i > 0; i--) cnt[par[nodes[i]]] += cnt[nodes[i]];
        long long gain = 0;
        for (int v : nodes) {
            best[v] = v == 1 ? 0 : best[par[v]] + (long long)(dep[v] - dep[par[v]]) * cnt[v];
            gain = max(gain, best[v]);
        }
        printf("%lld\n", total - gain);
    }
}
