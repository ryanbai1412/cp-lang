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
    Csr g(n, u, v, true);
    vector<int> tin(n, -1), low(n), children(n, 0), fu, fi, fp;
    vector<char> cut(n, 0);
    int timer = 0;
    for (int r = 0; r < n; r++) {
        if (tin[r] >= 0) continue;
        fu.push_back(r); fi.push_back(0); fp.push_back(-1);
        while (!fu.empty()) {
            int x = fu.back(), i = fi.back(), p = fp.back();
            if (i == 0) tin[x] = low[x] = timer++;
            if (g.start[x] + i < g.start[x + 1]) {
                fi.back()++;
                int y = g.to[g.start[x] + i];
                if (y == p) continue;
                if (tin[y] < 0) children[x]++, fu.push_back(y), fi.push_back(0), fp.push_back(x);
                else low[x] = min(low[x], tin[y]);
            } else {
                fu.pop_back(); fi.pop_back(); fp.pop_back();
                if (p < 0) { if (children[x] >= 2) cut[x] = 1; }
                else {
                    low[p] = min(low[p], low[x]);
                    if (fu.size() > 1 && low[x] >= tin[p]) cut[p] = 1;
                }
            }
        }
    }
    ll count = 0, total = 0;
    for (int x = 0; x < n; x++) if (cut[x]) count++, total += x;
    printf("%lld %lld\n", count, total);
}
