// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, q;
    cin >> n >> q;
    vector<array<int, 3>> c(q);
    vector<vector<int>> g(n + 1), rg(n + 1);
    for (auto &[t, u, v] : c) cin >> t >> u >> v, g[u].push_back(v), rg[v].push_back(u);
    vector<int> order, seen(n + 1, 0), comp(n + 1, -1);
    for (int s = 1; s <= n; s++) {
        if (seen[s]) continue;
        vector<pair<int, size_t>> st = {{s, 0}};
        seen[s] = 1;
        while (!st.empty()) {
            auto &[u, i] = st.back();
            if (i < g[u].size()) {
                int v = g[u][i++];
                if (!seen[v]) seen[v] = 1, st.push_back({v, 0});
            } else order.push_back(u), st.pop_back();
        }
    }
    int k = 0;
    for (int j = n - 1; j >= 0; j--) {
        int s = order[j];
        if (comp[s] >= 0) continue;
        vector<int> st = {s};
        comp[s] = k;
        while (!st.empty()) {
            int u = st.back();
            st.pop_back();
            for (int v : rg[u])
                if (comp[v] < 0) comp[v] = k, st.push_back(v);
        }
        k++;
    }
    for (auto &[t, u, v] : c)
        if (t == 1 && comp[u] == comp[v]) { cout << "No\n"; return 0; }
    sort(c.begin(), c.end(), [&](auto &a, auto &b) { return comp[a[1]] < comp[b[1]]; });
    vector<int> a(k, 1);
    for (auto &[t, u, v] : c)
        if (comp[u] != comp[v]) a[comp[v]] = max(a[comp[v]], a[comp[u]] + t);
    cout << "Yes\n";
    for (int v = 1; v <= n; v++) cout << a[comp[v]] << (v == n ? "\n" : " ");
}
