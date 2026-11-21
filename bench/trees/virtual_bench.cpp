// Matched C++ for virtual_bench.cp: the root_bench.cpp build plus chain-jumping lca, then per
// query sort keys by preorder, add adjacent lcas, sort and dedupe, and link with a stack.
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
    vector<int> top(n), tin(n);
    tin[0] = 0, top[0] = 0;
    for (int u : order) {
        int next = tin[u] + 1;
        if (heavy[u] >= 0) top[heavy[u]] = top[u], tin[heavy[u]] = next, next += sz[heavy[u]];
        for (int k = st[u]; k < st[u + 1]; k++) {
            int c = adj[k];
            if (c != par[u] && c != heavy[u]) top[c] = c, tin[c] = next, next += sz[c];
        }
    }
    auto lca = [&](int x, int y) {
        while (top[x] != top[y]) {
            if (dep[top[x]] < dep[top[y]]) swap(x, y);
            x = par[top[x]];
        }
        return dep[x] < dep[y] ? x : y;
    };
    auto is_anc = [&](int a, int b) { return tin[a] <= tin[b] && tin[b] < tin[a] + sz[a]; };
    auto by_tin = [&](int a, int b) { return tin[a] < tin[b]; };
    int q;
    if (scanf("%d", &q) != 1) return 1;
    long long hash = 0;
    vector<int> ks, vs, stk, up;
    vector<long long> len, cnt;
    vector<char> key;
    for (int qi = 0; qi < q; qi++) {
        int k;
        if (scanf("%d", &k) != 1) return 1;
        ks.resize(k);
        for (int i = 0; i < k; i++) {
            if (scanf("%d", &ks[i]) != 1) return 1;
            ks[i]--;
        }
        sort(ks.begin(), ks.end(), by_tin);
        ks.erase(unique(ks.begin(), ks.end()), ks.end());
        vs = ks;
        for (size_t i = 1; i < ks.size(); i++) vs.push_back(lca(ks[i - 1], ks[i]));
        sort(vs.begin(), vs.end(), by_tin);
        vs.erase(unique(vs.begin(), vs.end()), vs.end());
        int m = vs.size();
        up.assign(m, -1), len.assign(m, 0), key.assign(m, 0), cnt.assign(m, 0);
        stk.clear();
        for (int x = 0, j = 0; x < m; x++) {
            if (j < (int)ks.size() && ks[j] == vs[x]) key[x] = 1, j++;
            while (!stk.empty() && !is_anc(vs[stk.back()], vs[x])) stk.pop_back();
            if (!stk.empty()) up[x] = stk.back(), len[x] = dep[vs[x]] - dep[vs[up[x]]];
            stk.push_back(x);
        }
        long long sum = 0;
        for (int x = m - 1; x >= 0; x--) {
            if (key[x]) cnt[x]++;
            if (up[x] >= 0) sum += len[x] * cnt[x] * (k - cnt[x]), cnt[up[x]] += cnt[x];
        }
        hash = (hash * 31 + sum % 1000000007 + m) % 1000000007;
    }
    printf("%lld\n", hash);
}
