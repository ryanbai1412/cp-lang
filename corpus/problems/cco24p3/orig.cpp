// CCO 2024 Summer Driving. Root the tree at R. If A <= B Bob can always end at city 1. Otherwise
// every Alice turn starts at a node x whose root path is used (plus perhaps one used child edge
// c below x), so she drives exactly A edges down into subtree(x) outside branch c; Bob then ends
// within distance B of her endpoint v, every such end being strictly inside her previous subtree.
// Binary search the answer K: win means the game ends at a city >= K. With F(x) / G(x, c) Alice's
// win from x (no / child c excluded), Bob beats v iff some non-ancestor x' within distance B has
// F(x') = 0, or an ancestor y with j <= B edges up (child c toward v) has G(y, c) = 0. Both
// reduce to min-depth values per subtree plus a path minimum from v up its B ancestors, taken
// with a union-find in reverse preorder, when anc_B(v) is finished.
#include <bits/stdc++.h>
using namespace std;
const int INF = 1e9;
int n, R, A, B;
vector<int> par_, dep, order_, ancA, ancA1, ancB, adjStart, adjList, exc, exChild;
vector<vector<int>> byAncB;
vector<int> dsu, wmin;
int findMin(int v, int &mn) {
    // iterative find with path compression; wmin[v] = min edge key from v up to dsu[v]
    static vector<int> st; st.clear();
    while (dsu[v] != v) { st.push_back(v); v = dsu[v]; }
    int root = v;
    for (int i = (int)st.size() - 1; i >= 0; i--) {
        int x = st[i];
        if (dsu[x] != root) { wmin[x] = min(wmin[x], wmin[dsu[x]]); dsu[x] = root; }
    }
    mn = st.empty() ? INF : wmin[st[0]];
    return root;
}
bool wins(int K) {
    vector<int> ld(n + 1), dsub(n + 1), cnt(n + 1, 0), cntChild(n + 1, 0);
    vector<char> F(n + 1);
    for (int v = 1; v <= n; v++) { dsu[v] = v; wmin[v] = INF; }
    for (int idx = n - 1; idx >= 0; idx--) {
        int u = order_[idx];
        // children top-2 of ld (labels < K) and dsub (losing F)
        int l1 = INF, l2 = INF, l1c = -1, d1 = INF, d2 = INF, d1c = -1;
        for (int e = adjStart[u]; e < adjStart[u + 1]; e++) {
            int c = adjList[e]; if (c == par_[u]) continue;
            int lv = ld[c] + 1; if (lv < l1) { l2 = l1; l1 = lv; l1c = c; } else if (lv < l2) l2 = lv;
            int dv = dsub[c] + 1; if (dv < d1) { d2 = d1; d1 = dv; d1c = c; } else if (dv < d2) d2 = dv;
        }
        ld[u] = u < K ? 0 : min(l1, INF);
        auto finalWin = [&](int c) {  // Bob's final phase from u, branch c (or -1) excluded
            if (u < K) return false;
            int best = (c != -1 && c == l1c) ? l2 : l1;
            return best > B;
        };
        F[u] = exc[u] > 0 ? cnt[u] > 0 : finalWin(-1);
        dsub[u] = F[u] ? min(d1, INF) : 0;
        for (int e = adjStart[u]; e < adjStart[u + 1]; e++) {
            int c = adjList[e]; if (c == par_[u]) continue;
            bool g = exc[u] - exChild[c] > 0 ? cnt[u] - cntChild[c] > 0 : finalWin(c);
            int h = g ? (c == d1c ? d2 : d1) : 0;
            dsu[c] = u; wmin[c] = h >= INF ? INF : h - dep[u];
        }
        for (int v : byAncB[u]) {
            int mn; findMin(v, mn);
            bool bad = dsub[v] <= B || (mn < INF && mn <= B - dep[v]);
            if (!bad && ancA[v]) { cnt[ancA[v]]++; cntChild[ancA1[v]]++; }
        }
    }
    return F[R];
}
int main() {
    if (scanf("%d %d %d %d", &n, &R, &A, &B) != 4) return 0;
    vector<pair<int, int>> ed(n - 1);
    vector<int> deg(n + 2, 0);
    for (auto &[u, v] : ed) { if (scanf("%d %d", &u, &v) != 2) return 0; deg[u]++; deg[v]++; }
    if (A <= B) { puts("1"); return 0; }
    adjStart.assign(n + 2, 0);
    for (int v = 1; v <= n; v++) adjStart[v + 1] = adjStart[v] + deg[v];
    adjList.resize(2 * (n - 1));
    vector<int> pos(adjStart.begin(), adjStart.end());
    for (auto [u, v] : ed) { adjList[pos[u]++] = v; adjList[pos[v]++] = u; }
    par_.assign(n + 1, 0); dep.assign(n + 1, 0); ancA.assign(n + 1, 0); ancA1.assign(n + 1, 0);
    ancB.assign(n + 1, 0); exc.assign(n + 1, 0); exChild.assign(n + 1, 0); byAncB.assign(n + 1, {});
    // iterative DFS keeping the root path, for preorder and the A, A-1, B ancestors
    vector<int> path, it(n + 1);
    path.push_back(R); order_.push_back(R);
    for (int v = 1; v <= n; v++) it[v] = adjStart[v];
    while (!path.empty()) {
        int u = path.back();
        if (it[u] == adjStart[u + 1]) { path.pop_back(); continue; }
        int c = adjList[it[u]++];
        if (c == par_[u]) continue;
        par_[c] = u; dep[c] = dep[u] + 1; path.push_back(c); order_.push_back(c);
        int d = dep[c];
        if (d >= A) { ancA[c] = path[d - A]; ancA1[c] = path[d - A + 1]; exc[ancA[c]]++; exChild[ancA1[c]]++; }
        if (d >= B) { ancB[c] = path[d - B]; byAncB[ancB[c]].push_back(c); }
    }
    dsu.assign(n + 1, 0); wmin.assign(n + 1, INF);
    int lo = 1, hi = n;  // wins(lo) holds
    while (lo < hi) { int mid = (lo + hi + 1) / 2; if (wins(mid)) lo = mid; else hi = mid - 1; }
    printf("%d\n", lo);
}
