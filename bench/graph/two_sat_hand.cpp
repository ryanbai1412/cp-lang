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
// Implication graph, node 2x is "x is false" and 2x+1 "x is true"; Tarjan SCC.
int main() {
    int n = rd(), m = rd(), N = 2 * n;
    vector<vector<int>> imp(N);
    for (int i = 0; i < m; i++) {
        int a = 2 * rd(); a += rd();
        int b = 2 * rd(); b += rd();
        imp[a ^ 1].push_back(b); imp[b ^ 1].push_back(a);
    }
    vector<int> seen(N, -1), low(N), done(N, -1), stack, pu, pi;
    int timer = 0, k = 0;
    for (int r = 0; r < N; r++) {
        if (seen[r] >= 0) continue;
        pu.push_back(r); pi.push_back(0);
        while (!pu.empty()) {
            int x = pu.back(), i = pi.back();
            if (i == 0) { seen[x] = low[x] = timer++; stack.push_back(x); }
            if (i < (int)imp[x].size()) {
                pi.back()++;
                int y = imp[x][i];
                if (seen[y] < 0) pu.push_back(y), pi.push_back(0);
                else if (done[y] < 0) low[x] = min(low[x], seen[y]);
            } else {
                pu.pop_back(); pi.pop_back();
                if (low[x] == seen[x]) {
                    for (;;) { int y = stack.back(); stack.pop_back(); done[y] = k; if (y == x) break; }
                    k++;
                }
                if (!pu.empty()) low[pu.back()] = min(low[pu.back()], low[x]);
            }
        }
    }
    int count = 0;
    for (int x = 0; x < n; x++) {
        if (done[2 * x] == done[2 * x + 1]) { puts("-1"); return 0; }
        count += done[2 * x + 1] < done[2 * x];
    }
    printf("%d\n", count);
}
