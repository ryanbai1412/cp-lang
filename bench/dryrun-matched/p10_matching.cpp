#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    cin >> n;
    vector<vector<int>> g(n + 1);
    for (int i = 1; i < n; i++) {
        int a, b;
        cin >> a >> b;
        g[a].push_back(b), g[b].push_back(a);
    }
    vector<int> ord{1}, par(n + 1, 0);
    par[1] = -1;
    for (size_t i = 0; i < ord.size(); i++) {
        int u = ord[i];
        for (int v : g[u])
            if (v != par[u]) par[v] = u, ord.push_back(v);
    }
    vector<char> m(n + 1, 0);
    int ans = 0;
    for (int i = n - 1; i > 0; i--) {
        int u = ord[i];
        if (!m[u] && !m[par[u]]) m[u] = m[par[u]] = 1, ans++;
    }
    cout << ans << '\n';
}
