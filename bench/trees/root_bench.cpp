// Matched C++ for root_bench.cp: CSR adjacency, BFS order, then what Rooted keeps
// (parent, depth, size, heavy child, chain top, heavy-first preorder and its inverse).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    if (scanf("%d", &n) != 1) return 0;
    vector<int> eu(n - 1), ev(n - 1), deg(n + 1, 0);
    for (int i = 0; i < n - 1; i++) {
        if (scanf("%d %d", &eu[i], &ev[i]) != 2) return 1;
        eu[i]--, ev[i]--;
        deg[eu[i]]++, deg[ev[i]]++;
    }
    vector<int> st(n + 1, 0), adj(2 * (n - 1));
    for (int v = 0; v < n; v++) st[v + 1] = st[v] + deg[v];
    vector<int> fill(st.begin(), st.end() - 1);
    for (int i = 0; i < n - 1; i++) adj[fill[eu[i]]++] = ev[i], adj[fill[ev[i]]++] = eu[i];
    vector<int> par(n, -1), dep(n, 0), order;
    order.reserve(n);
    order.push_back(0);
    for (int i = 0; i < (int)order.size(); i++) {
        int u = order[i];
        for (int k = st[u]; k < st[u + 1]; k++) {
            int c = adj[k];
            if (c != par[u]) par[c] = u, dep[c] = dep[u] + 1, order.push_back(c);
        }
    }
    vector<int> sz(n, 1), heavy(n, -1);
    for (int i = n - 1; i > 0; i--) sz[par[order[i]]] += sz[order[i]];
    for (int u : order)
        for (int k = st[u]; k < st[u + 1]; k++) {
            int c = adj[k];
            if (c != par[u] && (heavy[u] < 0 || sz[c] > sz[heavy[u]])) heavy[u] = c;
        }
    vector<int> top(n), tin(n), at(n);
    tin[0] = 0, top[0] = 0;
    for (int u : order) {
        int next = tin[u] + 1;
        if (heavy[u] >= 0) top[heavy[u]] = top[u], tin[heavy[u]] = next, next += sz[heavy[u]];
        for (int k = st[u]; k < st[u + 1]; k++) {
            int c = adj[k];
            if (c != par[u] && c != heavy[u]) top[c] = c, tin[c] = next, next += sz[c];
        }
    }
    for (int v = 0; v < n; v++) at[tin[v]] = v;
    vector<long long> s(n);
    for (int v = 0; v < n; v++) s[v] = v;
    for (int p = n - 1; p >= 0; p--) {
        int v = at[p];
        if (par[v] >= 0) s[par[v]] += s[v];
    }
    long long hash = 0;
    for (int v = 0; v < n; v++) hash = (hash * 31 + s[v] % 1000003 * (tin[v] + 1) + dep[v]) % 1000000007;
    printf("%lld\n", hash);
}
