#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N = 100000;
vector<int> adj[N]; int mate[N], back_[N], level_[N], nx[N];
void layers() {
    vector<int> q; for (int u = 0; u < N; u++) { level_[u] = -1; if (mate[u] < 0) { level_[u] = 0; q.push_back(u); } }
    for (size_t at = 0; at < q.size(); at++) { int u = q[at]; for (int v : adj[u]) { int w = back_[v]; if (w >= 0 && level_[w] < 0) { level_[w] = level_[u] + 1; q.push_back(w); } } }
}
int main() {
    for (int i = 0; i < 400000; i++) { int u = nxt(N); adj[u].push_back(nxt(N)); }
    fill(mate, mate + N, -1); fill(back_, back_ + N, -1); int size = 0;
    for (int u = 0; u < N; u++) for (int v : adj[u]) if (back_[v] < 0) { mate[u] = v; back_[v] = u; size++; break; }
    while (true) {
        layers(); bool any = false;
        for (int u = 0; u < N && !any; u++) if (level_[u] >= 0) for (int v : adj[u]) if (back_[v] < 0) { any = true; break; }
        if (!any) break;
        fill(nx, nx + N, 0);
        for (int root = 0; root < N; root++) { if (mate[root] >= 0) continue;
            vector<int> path{root};
            while (!path.empty()) { int u = path.back();
                if (nx[u] == (int)adj[u].size()) { level_[u] = -1; path.pop_back(); continue; }
                int v = adj[u][nx[u]++]; int w = back_[v];
                if (w >= 0) { if (level_[w] == level_[u] + 1) path.push_back(w); }
                else { int fr = v; for (int i = path.size() - 1; i >= 0; i--) { int x = path[i]; int old = mate[x]; mate[x] = fr; back_[fr] = x; if (old >= 0) fr = old; } size++; path.clear(); } } }
    }
    printf("%d\n", size);
}
