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
// Hierholzer with an explicit stack of (vertex, arc that led there).
int main() {
    int n = rd(), m = rd();
    vector<int> u, v; read_edges(m, u, v);
    Csr g(n, u, v, true);
    int start = -1;
    for (int x = 0; x < n && start < 0; x++) if ((g.start[x + 1] - g.start[x]) % 2) start = x;
    for (int x = 0; x < n && start < 0; x++) if (g.start[x + 1] > g.start[x]) start = x;
    vector<int> at(g.start.begin(), g.start.end() - 1), sv{start}, sa{-1}, arcs;
    vector<char> used(m, 0);
    while (!sv.empty()) {
        int x = sv.back();
        while (at[x] < g.start[x + 1] && used[g.e[at[x]]]) at[x]++;
        if (at[x] < g.start[x + 1]) {
            int a = at[x];
            used[g.e[a]] = 1;
            sv.push_back(g.to[a]); sa.push_back(a);
        } else {
            if (sa.back() >= 0) arcs.push_back(sa.back());
            sv.pop_back(); sa.pop_back();
        }
    }
    if ((int)arcs.size() != m) return 1;
    reverse(arcs.begin(), arcs.end());
    ll total = 0;
    for (size_t i = 0; i < arcs.size(); i++) total += (ll)(i + 1) * g.e[arcs[i]];
    printf("%d %lld\n", start, total);
}
