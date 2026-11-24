// noimps23p2: E' is the edge set of a connected set of blocks (biconnected components) and V'
// all their vertices; each vertex of V' keeps the parts of the graph hanging off it. Root the
// block-cut tree at its centroid (by vertex count), which every valid choice contains. For a
// least component size s, a block with more than s vertices below it is chosen and one with
// fewer is not, so only the blocks with exactly s below are free, and the count is a product
// over the O(n / s) vertices that are surely in V'; summed over s it is O(n log n).
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const ll MOD = 998244353;

int n, m, K, N;
vector<vector<int>> T;  // block-cut tree: vertices 0..n-1, blocks n..N-1
vector<int> sz, par, rounds, key;  // key: vertex count below a vertex's parent block
vector<vector<int>> kids;  // a vertex's child blocks, by size descending

void blocks(const vector<vector<pair<int, int>>>& g) {
    vector<int> dfn(n, -1), low(n), st, it(n);
    int timer = 0;
    T.assign(n, {});
    vector<pair<int, int>> stk;  // (vertex, parent edge)
    dfn[0] = low[0] = timer++;
    stk.push_back({0, -1});
    st.push_back(0);
    while (!stk.empty()) {
        auto [v, pe] = stk.back();
        if (it[v] < (int)g[v].size()) {
            auto [w, id] = g[v][it[v]++];
            if (id == pe) continue;
            if (dfn[w] < 0) {
                dfn[w] = low[w] = timer++;
                st.push_back(w);
                stk.push_back({w, id});
            } else low[v] = min(low[v], dfn[w]);
            continue;
        }
        stk.pop_back();
        if (stk.empty()) break;
        int u = stk.back().first;
        low[u] = min(low[u], low[v]);
        if (low[v] >= dfn[u]) {
            int b = T.size();
            T.push_back({});
            while (true) {
                int x = st.back();
                st.pop_back();
                T[b].push_back(x), T[x].push_back(b);
                if (x == v) break;
            }
            T[b].push_back(u), T[u].push_back(b);
        }
    }
    N = T.size();
}

void root(int r) {
    sz.assign(N, 0), par.assign(N, -1);
    vector<int> order = {r};
    for (size_t i = 0; i < order.size(); i++)
        for (int w : T[order[i]])
            if (w != par[order[i]]) par[w] = order[i], order.push_back(w);
    for (int i = N - 1; i >= 0; i--) {
        int v = order[i];
        sz[v] += v < n;
        if (par[v] >= 0) sz[par[v]] += sz[v];
    }
}

ll count(int s, bool loose) {
    // loose: component sizes in {s, s + 1}; else all exactly s
    int lim = loose ? s + 1 : s;
    ll ways = 1;
    size_t cnt = 0;
    for (int u : rounds) {
        if (key[u] < lim) break;
        if (++cnt > (size_t)(n / s + 1)) return 0;
        ll F = sz[u] - 1, A = 0, B = 0;
        for (int q : kids[u]) {
            if (sz[q] < s) break;
            F -= sz[q];
            if (sz[q] == s && loose) A++, B += T[q].size() != 2;
        }
        if (!loose) {
            if (1 + F != s) return 0;
            continue;
        }
        ll w = (B == 0 && (1 + F == s || 1 + F == s + 1)) + (F == 0 ? (B == 0 ? A : B == 1) : 0);
        ways = ways * (w % MOD) % MOD;
        if (!ways) return 0;
    }
    return ways;
}

int main() {
    scanf("%d %d %d", &n, &m, &K);
    vector<vector<pair<int, int>>> g(n);
    for (int i = 0; i < m; i++) {
        int u, v;
        scanf("%d %d", &u, &v);
        u--, v--;
        g[u].push_back({v, i}), g[v].push_back({u, i});
    }
    blocks(g);
    root(0);
    int c = 0;
    for (int v = 0; v < N; v++) {
        int big = n - sz[v];
        for (int w : T[v])
            if (w != par[v]) big = max(big, sz[w]);
        if (2 * big <= n) { c = v; break; }
    }
    root(c);
    key.assign(n, 0), kids.assign(n, {});
    for (int u = 0; u < n; u++) {
        key[u] = u == c ? INT_MAX : par[u] == c ? n : sz[par[u]];
        for (int q : T[u])
            if (q != par[u]) kids[u].push_back(q);
        sort(kids[u].begin(), kids[u].end(), [&](int a, int b) { return sz[a] > sz[b]; });
        rounds.push_back(u);
    }
    sort(rounds.begin(), rounds.end(), [&](int a, int b) { return key[a] > key[b]; });
    ll ans = 0;
    for (int s = 1; 2 * s <= n; s++) {
        if (!K) ans += count(s, false);
        else ans += count(s, true) - (2 * (s + 1) <= n ? count(s + 1, false) : 0);
        ans %= MOD;
    }
    printf("%lld\n", (ans % MOD + MOD) % MOD);
}
