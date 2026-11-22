#include <bits/stdc++.h>
using namespace std;
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, m;
    cin >> n >> m;
    vector<vector<int>> g(n + 1);
    vector<int> in(n + 1);
    for (int i = 0; i < m; i++) {
        int a, b;
        cin >> a >> b;
        g[a].push_back(b), in[b]++;
    }
    priority_queue<int, vector<int>, greater<int>> q;
    for (int i = 1; i <= n; i++)
        if (!in[i]) q.push(i);
    vector<int> o;
    while (!q.empty()) {
        int u = q.top();
        q.pop();
        o.push_back(u);
        for (int v : g[u])
            if (--in[v] == 0) q.push(v);
    }
    if ((int)o.size() < n) {
        cout << "IMPOSSIBLE\n";
        return 0;
    }
    for (int x : o) cout << x << ' ';
    cout << '\n';
}
