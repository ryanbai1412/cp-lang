// sdoi17r2p3 (SDOI 2017 Cut Tree Game) reference. In the Walsh-Hadamard domain the xor
// convolution is pointwise, so with f_u the transform of the subtrees whose top is u,
// f_u = g_u prod_c (1 + f_c) and the answer is the transform of S = sum_u f_u. Dynamic DP over
// heavy paths: with L_u = g_u times the light factors and LS_u the light children's subtree
// sums, (f_u, s_u, 1) = M_u (f_h, s_h, 1), M_u = [[L,0,L],[L,1,L+LS],[0,0,1]], kept as
// (a,b,c,d) in a segment tree per heavy path. Light factors can be 0 mod 10007, so each node
// keeps the product of the nonzero ones and a zero count. O((n + changes log^2 n) m).
#include <bits/stdc++.h>
using namespace std;
typedef uint16_t u16;
const int MOD = 10007;

int n, M;
vector<vector<int>> adj;
vector<int> par, heavy, top, sz, val, inv;
vector<vector<int>> chainOf;  // nodes of the path, top first, indexed by its top
vector<int> posIn, segBase;
vector<u16> seg;  // per segment-tree node: a, b, c, d, each M values
vector<vector<u16>> nz, zc, LS, F, S;  // F, S cached at path tops

u16* node(int t, int idx) { return &seg[(size_t)(segBase[t] + idx) * 4 * M]; }

int g(int u, int i) { return __builtin_popcount(i & val[u]) & 1 ? MOD - 1 : 1; }

void leafMat(int u, u16* p) {
    for (int i = 0; i < M; i++) {
        int L = zc[u][i] ? 0 : g(u, i) * nz[u][i] % MOD;
        p[i] = p[M + i] = p[2 * M + i] = L;
        p[3 * M + i] = (L + LS[u][i]) % MOD;
    }
}

void combine(u16* p, const u16* x, const u16* y) {  // x above y
    for (int i = 0; i < M; i++) {
        int a1 = x[i], b1 = x[M + i], c1 = x[2 * M + i], d1 = x[3 * M + i];
        int a2 = y[i], b2 = y[M + i], c2 = y[2 * M + i], d2 = y[3 * M + i];
        p[i] = a1 * a2 % MOD;
        p[M + i] = (a1 * b2 + b1) % MOD;
        p[2 * M + i] = (c1 * a2 + c2) % MOD;
        p[3 * M + i] = (c1 * b2 + d1 + d2) % MOD;
    }
}

void build(int t, int idx, int l, int r) {
    if (l == r) { leafMat(chainOf[t][l], node(t, idx)); return; }
    int mid = (l + r) / 2, lc = idx + 1, rc = idx + 2 * (mid - l + 1);
    build(t, lc, l, mid), build(t, rc, mid + 1, r);
    combine(node(t, idx), node(t, lc), node(t, rc));
}

void update(int t, int idx, int l, int r, int pos) {
    if (l == r) { leafMat(chainOf[t][l], node(t, idx)); return; }
    int mid = (l + r) / 2, lc = idx + 1, rc = idx + 2 * (mid - l + 1);
    if (pos <= mid) update(t, lc, l, mid, pos);
    else update(t, rc, mid + 1, r, pos);
    combine(node(t, idx), node(t, lc), node(t, rc));
}

void addLight(int p, int t, int sign) {  // sign +1 adds path top t's factor to p, -1 removes it
    for (int i = 0; i < M; i++) {
        int o = (1 + F[t][i]) % MOD;
        if (sign > 0) {
            if (o) nz[p][i] = nz[p][i] * o % MOD; else zc[p][i]++;
            LS[p][i] = (LS[p][i] + S[t][i]) % MOD;
        } else {
            if (o) nz[p][i] = nz[p][i] * inv[o] % MOD; else zc[p][i]--;
            LS[p][i] = (LS[p][i] + MOD - S[t][i]) % MOD;
        }
    }
}

void readTop(int t) {
    u16* r = node(t, 0);
    for (int i = 0; i < M; i++) F[t][i] = r[M + i], S[t][i] = r[3 * M + i];
}

int main() {
    int m;
    scanf("%d %d", &n, &m);
    M = 1;
    while (M < m) M <<= 1;
    inv.assign(MOD, 1);
    for (int i = 2; i < MOD; i++) inv[i] = (MOD - (MOD / i) * inv[MOD % i] % MOD) % MOD;
    val.resize(n), adj.assign(n, {});
    for (auto& v : val) scanf("%d", &v);
    for (int i = 0; i + 1 < n; i++) {
        int a, b;
        scanf("%d %d", &a, &b);
        a--, b--;
        adj[a].push_back(b), adj[b].push_back(a);
    }
    par.assign(n, -1), heavy.assign(n, -1), sz.assign(n, 1), top.assign(n, 0);
    vector<int> order = {0};
    for (size_t i = 0; i < order.size(); i++)
        for (int v : adj[order[i]])
            if (v != par[order[i]]) par[v] = order[i], order.push_back(v);
    for (int i = n - 1; i > 0; i--) sz[par[order[i]]] += sz[order[i]];
    for (int u : order)
        for (int v : adj[u])
            if (v != par[u] && (heavy[u] < 0 || sz[v] > sz[heavy[u]])) heavy[u] = v;
    chainOf.assign(n, {}), posIn.assign(n, 0), segBase.assign(n, 0);
    int total = 0;
    for (int u : order)
        if (u == 0 || heavy[par[u]] != u) {
            for (int w = u; w >= 0; w = heavy[w]) top[w] = u, posIn[w] = chainOf[u].size(), chainOf[u].push_back(w);
            segBase[u] = total, total += 2 * chainOf[u].size() - 1;
        }
    seg.assign((size_t)total * 4 * M, 0);
    auto vec = [&](int x) { return vector<vector<u16>>(n, vector<u16>(M, x)); };
    nz = vec(1), zc = vec(0), LS = vec(0), F = vec(0), S = vec(0);
    for (int i = n - 1; i >= 0; i--) {  // deeper paths first
        int u = order[i];
        if (top[u] != u) continue;
        build(u, 0, 0, chainOf[u].size() - 1);
        readTop(u);
        if (u) addLight(par[u], u, +1);
    }
    int invM = inv[M % MOD];
    vector<int> ans;
    bool fresh = false;
    int q;
    scanf("%d", &q);
    char op[16];
    while (q--) {
        scanf("%s", op);
        if (op[0] == 'C') {
            int x, y;
            scanf("%d %d", &x, &y);
            x--;
            val[x] = y;
            for (int u = x;;) {
                int t = top[u];
                if (t) addLight(par[t], t, -1);
                update(t, 0, 0, chainOf[t].size() - 1, posIn[u]);
                readTop(t);
                if (!t) break;
                addLight(par[t], t, +1);
                u = par[t];
            }
            fresh = false;
        } else {
            int k;
            scanf("%d", &k);
            if (!fresh) {
                ans.assign(S[0].begin(), S[0].end());
                for (int len = 1; len < M; len <<= 1)
                    for (int i = 0; i < M; i += 2 * len)
                        for (int j = i; j < i + len; j++) {
                            int x = ans[j], y = ans[j + len];
                            ans[j] = (x + y) % MOD, ans[j + len] = (x - y + MOD) % MOD;
                        }
                for (auto& x : ans) x = x * invM % MOD;
                fresh = true;
            }
            printf("%d\n", ans[k]);
        }
    }
}
