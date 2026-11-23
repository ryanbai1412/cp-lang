#include <bits/stdc++.h>
using namespace std;
int main() {
    int t; scanf("%d", &t);
    vector<int> deg;
    while (t--) {
        int n, m; scanf("%d %d", &n, &m);
        deg.assign(n + 1, 0);
        for (int i = 0; i < m; i++) { int u, v; scanf("%d %d", &u, &v); deg[u]++; deg[v]++; }
        bool iso = false; int mx = 0;
        for (int v = 1; v <= n; v++) { iso |= deg[v] == 0; mx = max(mx, deg[v]); }
        puts(iso ? "Brain Melt" : (m == n - 1 && mx == n - 1) ? "2" : "Many");
    }
}
