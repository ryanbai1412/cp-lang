// Not a judge-accepted submission: written for the corpus and stress-checked against the port and its brute.
#include <bits/stdc++.h>
using namespace std;
const long long MOD = 998244353;
int main() {
    int n, m;
    cin >> n >> m;
    vector<vector<pair<int, int>>> adj(n + 1);
    for (int i = 0; i < m; i++) {
        int u, v, w;
        cin >> u >> v >> w;
        adj[u].push_back({v, w});
    }
    vector<long long> val(n + 1, -1);
    val[1] = 0;
    vector<vector<int>> layer = {{1}};
    while (!layer.empty()) {
        vector<vector<int>> next;
        for (auto &g : layer) {
            long long here = val[g[0]];
            vector<vector<int>> by(10);
            for (int u : g)
                for (auto [v, w] : adj[u])
                    if (val[v] < 0) by[w].push_back(v);
            for (int d = 1; d <= 9; d++) {
                vector<int> fresh;
                for (int v : by[d])
                    if (val[v] < 0) val[v] = (here * 10 + d) % MOD, fresh.push_back(v);
                if (!fresh.empty()) next.push_back(fresh);
            }
        }
        layer = next;
    }
    for (int v = 2; v <= n; v++) cout << val[v] << "\n";
}
