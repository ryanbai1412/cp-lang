// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    cin >> n;
    vector<vector<pair<int, int>>> adj(n + 1);
    for (int i = 1; i < n; i++) {
        int u, v;
        cin >> u >> v;
        adj[u].push_back({v, i});
        adj[v].push_back({u, i});
    }
    vector<int> par(n + 1, 0), up(n + 1, 0), order, sz(n + 1, 1);
    vector<int> st = {1};
    par[1] = -1;
    while (!st.empty()) {
        int u = st.back();
        st.pop_back();
        order.push_back(u);
        for (auto [v, i] : adj[u])
            if (v != par[u]) par[v] = u, up[v] = i, st.push_back(v);
    }
    for (int k = n - 1; k > 0; k--) sz[par[order[k]]] += sz[order[k]];
    long long base = 0;
    vector<long long> down(n + 1, 0), x(n + 1, 0);
    for (int c = 2; c <= n; c++) {
        if (sz[c] > n - sz[c]) down[c] += up[c];
        else base += up[c], down[c] -= up[c];
    }
    for (int u : order) x[u] = down[u] + (u == 1 ? base : x[par[u]]);
    for (int v = 1; v <= n; v++) cout << x[v] << (v == n ? "\n" : " ");
}
