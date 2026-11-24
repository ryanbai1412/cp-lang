// NOI 2023 P3 (Depth First Search), written for the corpus: no accepted C++ exists. T is an s-dfs
// tree of G exactly when every chosen non-tree edge joins an ancestor and a descendant of s, so the
// answer is the inclusion-exclusion sum over non-empty sets S of special roots of
// (-1)^(|S|+1) 2^(edges fine for every root in S). Rooted at 1, sets are grouped by z = lca(S):
// h(z) sums them over the vertical edges inside z's subtree, and a segment tree over dfs order keeps
// h(z) / 2^(edges below z) halved once for every edge above z that leaves the path to z, so a parent
// reads its children's totals with range sums. Cross edges are fine only when S sits under their two
// ends with lca(S) = their lca, which a sweep over the rectangles sub(a) x sub(b) adds at that lca.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
const ll MOD = 1e9 + 7, INV2 = (MOD + 1) / 2;
const int LOG = 20;

static char buf[1 << 25];
int bl = 0, bp = 0;
static inline int gc() {
    if (bp == bl) { bl = fread(buf, 1, sizeof buf, stdin); bp = 0; if (bl <= 0) return -1; }
    return buf[bp++];
}
static inline int rd() {
    int c = gc(), x = 0;
    while (c != -1 && (c < '0' || c > '9')) c = gc();
    while (c >= '0' && c <= '9') x = x * 10 + (c - '0'), c = gc();
    return x;
}

int n;
vector<ll> sv, sx, lz;
static void apply(int x, ll k) { sv[x] = sv[x] * k % MOD; sx[x] = sx[x] * k % MOD; lz[x] = lz[x] * k % MOD; }
static void push(int x) { if (lz[x] != 1) apply(2 * x, lz[x]), apply(2 * x + 1, lz[x]), lz[x] = 1; }
static void pull(int x) { sv[x] = (sv[2 * x] + sv[2 * x + 1]) % MOD; sx[x] = (sx[2 * x] + sx[2 * x + 1]) % MOD; }
static void mul(int x, int l, int r, int ql, int qr, ll k) {
    if (qr < l || r < ql || ql > qr) return;
    if (ql <= l && r <= qr) { apply(x, k); return; }
    push(x); int mid = (l + r) / 2;
    mul(2 * x, l, mid, ql, qr, k); mul(2 * x + 1, mid + 1, r, ql, qr, k); pull(x);
}
static void add(int x, int l, int r, int p, ll v, ll e) {
    if (l == r) { sv[x] = (sv[x] + v) % MOD; sx[x] = (sx[x] + e) % MOD; return; }
    push(x); int mid = (l + r) / 2;
    if (p <= mid) add(2 * x, l, mid, p, v, e); else add(2 * x + 1, mid + 1, r, p, v, e);
    pull(x);
}
static ll sum(int x, int l, int r, int ql, int qr) {
    if (qr < l || r < ql || ql > qr) return 0;
    if (ql <= l && r <= qr) return sv[x];
    push(x); int mid = (l + r) / 2;
    return (sum(2 * x, l, mid, ql, qr) + sum(2 * x + 1, mid + 1, r, ql, qr)) % MOD;
}
static void leaves(int x, int l, int r, vector<ll>& out) {
    if (l == r) { out[l] = (sv[x] + sx[x]) % MOD; return; }
    push(x); int mid = (l + r) / 2;
    leaves(2 * x, l, mid, out); leaves(2 * x + 1, mid + 1, r, out);
}

// The sweep's small tree over y blocks: range multiply, total.
struct Small {
    int sz; vector<ll> s, z;
    void init(const vector<ll>& w) {
        sz = w.size(); s.assign(4 * sz, 0); z.assign(4 * sz, 1); build(1, 0, sz - 1, w);
    }
    void build(int x, int l, int r, const vector<ll>& w) {
        if (l == r) { s[x] = w[l]; return; }
        int mid = (l + r) / 2; build(2 * x, l, mid, w); build(2 * x + 1, mid + 1, r, w);
        s[x] = (s[2 * x] + s[2 * x + 1]) % MOD;
    }
    void mul(int x, int l, int r, int ql, int qr, ll k) {
        if (qr < l || r < ql) return;
        if (ql <= l && r <= qr) { s[x] = s[x] * k % MOD; z[x] = z[x] * k % MOD; return; }
        int mid = (l + r) / 2;
        mul(2 * x, l, mid, ql, qr, k); mul(2 * x + 1, mid + 1, r, ql, qr, k);
        s[x] = (s[2 * x] + s[2 * x + 1]) % MOD * z[x] % MOD;
    }
};

int main() {
    rd(); n = rd(); int m = rd(), k = rd();
    vector<int> eu(n - 1), ev(n - 1), deg(n + 1, 0);
    for (int i = 0; i < n - 1; i++) eu[i] = rd(), ev[i] = rd(), deg[eu[i]]++, deg[ev[i]]++;
    vector<int> start(n + 2, 0), adj(2 * (n - 1));
    for (int v = 1; v <= n; v++) start[v + 1] = start[v] + deg[v];
    vector<int> fill(start.begin(), start.end());
    for (int i = 0; i < n - 1; i++) adj[fill[eu[i]]++] = ev[i], adj[fill[ev[i]]++] = eu[i];
    vector<int> par(n + 1, 0), dep(n + 1, 0), tin(n + 1), sz(n + 1, 1), order;
    order.reserve(n);
    {
        vector<int> st = {1}; par[1] = 0;
        while (!st.empty()) {
            int u = st.back(); st.pop_back();
            tin[u] = order.size(); order.push_back(u);
            for (int i = start[u]; i < start[u + 1]; i++)
                if (adj[i] != par[u]) par[adj[i]] = u, dep[adj[i]] = dep[u] + 1, st.push_back(adj[i]);
        }
        for (int i = n - 1; i > 0; i--) sz[par[order[i]]] += sz[order[i]];
    }
    vector<array<int, LOG>> up(n + 1);
    for (int u : order) {
        up[u][0] = par[u] ? par[u] : u;
        for (int j = 1; j < LOG; j++) up[u][j] = up[up[u][j - 1]][j - 1];
    }
    auto lift = [&](int u, int d) { for (int j = 0; d; j++, d >>= 1) if (d & 1) u = up[u][j]; return u; };
    auto anc = [&](int a, int b) { return tin[a] <= tin[b] && tin[b] < tin[a] + sz[a]; };
    auto lca = [&](int a, int b) {
        if (anc(a, b)) return a;
        if (anc(b, a)) return b;
        for (int j = LOG - 1; j >= 0; j--) if (!anc(up[a][j], b)) a = up[a][j];
        return par[a];
    };
    vector<vector<pair<int, int>>> vert(n + 1);           // at the upper end a: (c, b)
    vector<vector<array<int, 4>>> rects(n + 1);           // at the lca: sub(a) x sub(b)
    vector<int> to_child(n + 1, 0), cross_end(n + 1, 0);
    ll M_vert = 0;
    for (int i = 0; i < m; i++) {
        int a = rd(), b = rd();
        if (anc(b, a)) swap(a, b);
        if (anc(a, b)) {
            int c = lift(b, dep[b] - dep[a] - 1);
            vert[a].push_back({c, b}); to_child[c]++; M_vert++;
        } else {
            if (tin[a] > tin[b]) swap(a, b);
            rects[lca(a, b)].push_back({tin[a], tin[a] + sz[a] - 1, tin[b], tin[b] + sz[b] - 1});
            cross_end[a]++; cross_end[b]++;
        }
    }
    vector<char> special(n + 1, 0);
    for (int i = 0; i < k; i++) special[rd()] = 1;
    int top = max(n + m + 5, 2);
    vector<ll> pw(top), ipw(top);
    pw[0] = ipw[0] = 1;
    for (int i = 1; i < top; i++) pw[i] = pw[i - 1] * 2 % MOD, ipw[i] = ipw[i - 1] * INV2 % MOD;
    vector<int> ein(n + 1, 0);
    for (int i = n - 1; i >= 0; i--) {
        int u = order[i];
        ein[u] += vert[u].size();
        if (par[u]) ein[par[u]] += ein[u];
    }
    int S = 1; while (S < n) S <<= 1;
    sv.assign(2 * S, 0); sx.assign(2 * S, 0); lz.assign(2 * S, 1);
    for (int i = n - 1; i >= 0; i--) {
        int z = order[i];
        for (auto [c, b] : vert[z]) {
            mul(1, 0, S - 1, tin[c], tin[b] - 1, INV2);
            mul(1, 0, S - 1, tin[b] + sz[b], tin[c] + sz[c] - 1, INV2);
        }
        ll prodAB = 1, prodA = 1, sumB = 0;
        for (int j = start[z]; j < start[z + 1]; j++) {
            int y = adj[j];
            if (y == par[z]) continue;
            ll A = pw[ein[y] + to_child[y]], s = sum(1, 0, S - 1, tin[y], tin[y] + sz[y] - 1);
            prodAB = prodAB * ((A + A * s) % MOD) % MOD;
            prodA = prodA * A % MOD;
            sumB = (sumB + s) % MOD;
        }
        ll h = ((special[z] ? 0 : prodAB) - prodA - prodA * sumB % MOD + 2 * MOD) % MOD;
        ll extra = 0;
        if (!rects[z].empty()) {
            vector<int> ys;
            for (auto& r : rects[z]) ys.push_back(r[2]), ys.push_back(r[3] + 1);
            sort(ys.begin(), ys.end()); ys.erase(unique(ys.begin(), ys.end()), ys.end());
            vector<ll> w(ys.size() - 1);
            ll plain = 0;
            for (size_t j = 0; j + 1 < ys.size(); j++) w[j] = sum(1, 0, S - 1, ys[j], ys[j + 1] - 1), plain = (plain + w[j]) % MOD;
            Small t; t.init(w);
            vector<tuple<int, ll, int, int>> ev;
            for (auto& r : rects[z]) {
                int lo = lower_bound(ys.begin(), ys.end(), r[2]) - ys.begin();
                int hi = lower_bound(ys.begin(), ys.end(), r[3] + 1) - ys.begin() - 1;
                ev.push_back({r[0], 2, lo, hi}); ev.push_back({r[1] + 1, INV2, lo, hi});
            }
            sort(ev.begin(), ev.end());
            for (size_t j = 0; j < ev.size();) {
                int x = get<0>(ev[j]);
                for (; j < ev.size() && get<0>(ev[j]) == x; j++) t.mul(1, 0, t.sz - 1, get<2>(ev[j]), get<3>(ev[j]), get<1>(ev[j]));
                if (j < ev.size())
                    extra = (extra + sum(1, 0, S - 1, x, get<0>(ev[j]) - 1) * ((t.s[1] - plain + MOD) % MOD)) % MOD;
            }
            extra = extra * prodA % MOD;
        }
        add(1, 0, S - 1, tin[z], h * ipw[ein[z]] % MOD, extra * ipw[ein[z]] % MOD);
    }
    vector<ll> val(S);
    leaves(1, 0, S - 1, val);
    vector<int> fixed(n + 1, 0);
    ll total = 0;
    for (int u : order) {
        fixed[u] = cross_end[u] + (par[u] ? fixed[par[u]] : 0);
        total = (total + pw[fixed[u]] * val[tin[u]]) % MOD;
    }
    printf("%lld\n", (MOD - pw[M_vert] * total % MOD) % MOD);
}
