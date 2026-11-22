#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n;
    cin >> n;
    vector<int> eu(n - 1), ev(n - 1), start(n + 2, 0);
    for (int i = 0; i < n - 1; i++) {
        cin >> eu[i] >> ev[i];
        start[eu[i] + 1]++, start[ev[i] + 1]++;
    }
    for (int v = 1; v <= n + 1; v++) start[v] += start[v - 1];
    vector<int> adj(2 * (n - 1)), fill(start.begin(), start.end() - 1);
    for (int i = 0; i < n - 1; i++) adj[fill[eu[i]]++] = ev[i], adj[fill[ev[i]]++] = eu[i];
    vector<int> ord{1}, par(n + 1, 0);
    ord.reserve(n);
    for (size_t i = 0; i < ord.size(); i++) {
        int u = ord[i];
        for (int k = start[u]; k < start[u + 1]; k++)
            if (adj[k] != par[u]) par[adj[k]] = u, ord.push_back(adj[k]);
    }
    // per vertex: (free, used) = best with it unmatched / matched to a child, folded over the
    // children as (sum of bests, best gain of matching one), as the port's `t.up` does
    const long long NEG = LLONG_MIN / 4;
    vector<long long> s0(n + 1, 0), s1(n + 1, NEG), f0(n + 1), f1(n + 1);
    for (int i = n - 1; i >= 0; i--) {
        int u = ord[i];
        f0[u] = s0[u], f1[u] = s0[u] + s1[u];
        if (i == 0) break;
        int p = par[u];
        long long best = max(f0[u], f1[u]);
        s0[p] += best;
        s1[p] = max(s1[p], f0[u] + 1 - best);
    }
    cout << max(f0[1], f1[1]) << '\n';
}
