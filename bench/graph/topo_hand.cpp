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
int main() {
    int n = rd(), m = rd();
    vector<int> u, v; read_edges(m, u, v);
    Csr g(n, u, v, false);
    vector<int> indeg(n, 0), order; order.reserve(n);
    for (int i = 0; i < m; i++) indeg[v[i]]++;
    for (int x = 0; x < n; x++) if (indeg[x] == 0) order.push_back(x);
    for (size_t h = 0; h < order.size(); h++) {
        int x = order[h];
        for (int i = g.start[x]; i < g.start[x + 1]; i++) if (--indeg[g.to[i]] == 0) order.push_back(g.to[i]);
    }
    if ((int)order.size() != n) return 1;
    ll total = 0;
    for (size_t i = 0; i < order.size(); i++) total += (ll)(i + 1) * order[i];
    printf("%lld\n", total);
}
