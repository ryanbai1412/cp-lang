#include <bits/stdc++.h>
using namespace std;
using ll = long long;
static char buf[1 << 25];
static int bl = 0, bp = 0;
static inline int gc() {
    if (bp == bl) { bl = fread(buf, 1, sizeof buf, stdin); bp = 0; if (bl <= 0) return -1; }
    return buf[bp++];
}
static inline ll rd() {
    int c = gc(); while (c != '-' && (c < '0' || c > '9')) c = gc();
    ll x = 0; while (c >= '0' && c <= '9') { x = x * 10 + (c - '0'); c = gc(); }
    return x;
}
// Arcs out of each vertex in edge order, as `Graph` lays them out.
struct Csr {
    vector<int> start, to, e;
    Csr(int n, const vector<int>& u, const vector<int>& v, bool both)
        : start(n + 1, 0), to(u.size() * (both ? 2 : 1)), e(to.size()) {
        for (size_t i = 0; i < u.size(); i++) { start[u[i] + 1]++; if (both) start[v[i] + 1]++; }
        for (int i = 0; i < n; i++) start[i + 1] += start[i];
        vector<int> at(start.begin(), start.end() - 1);
        for (size_t i = 0; i < u.size(); i++) {
            to[at[u[i]]] = v[i]; e[at[u[i]]++] = i;
            if (both) { to[at[v[i]]] = u[i]; e[at[v[i]]++] = i; }
        }
    }
};
static void read_edges(int m, vector<int>& u, vector<int>& v) {
    u.resize(m); v.resize(m);
    for (int i = 0; i < m; i++) u[i] = rd() - 1, v[i] = rd() - 1;
}
// Heavy-light decomposition from a BFS order; LCA by climbing chains.
int main() {
    int n = rd(), q = rd();
    vector<int> u, v; read_edges(n - 1, u, v);
    Csr g(n, u, v, true);
    vector<int> par(n, -1), dep(n, 0), order; order.reserve(n);
    vector<char> seen(n, 0);
    seen[0] = 1; order.push_back(0);
    for (size_t h = 0; h < order.size(); h++) {
        int x = order[h];
        for (int i = g.start[x]; i < g.start[x + 1]; i++) {
            int y = g.to[i];
            if (!seen[y]) seen[y] = 1, par[y] = x, dep[y] = dep[x] + 1, order.push_back(y);
        }
    }
    vector<int> sz(n, 1), heavy(n, -1), top(n, 0);
    for (int i = n - 1; i > 0; i--) sz[par[order[i]]] += sz[order[i]];
    for (int i = 1; i < n; i++) {
        int c = order[i], p = par[c];
        if (heavy[p] < 0 || sz[c] > sz[heavy[p]]) heavy[p] = c;
    }
    for (int x : order) top[x] = par[x] >= 0 && heavy[par[x]] == x ? top[par[x]] : x;
    ll total = 0;
    while (q--) {
        int a = rd() - 1, b = rd() - 1;
        while (top[a] != top[b]) {
            if (dep[top[a]] < dep[top[b]]) swap(a, b);
            a = par[top[a]];
        }
        total += dep[a] < dep[b] ? a : b;
    }
    printf("%lld\n", total);
}
