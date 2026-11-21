// The same algorithms as tree_bench.cp, written directly: heavy-first preorder with chain-jump
// LCA, rerooting with prefix/suffix sibling folds, BFS-based centroid decomposition.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
static char buf[1 << 25];
static int buf_len = 0, buf_pos = 0;
static int rd() {
    auto get = [&]() -> int {
        if (buf_pos == buf_len) buf_len = fread(buf, 1, sizeof buf, stdin), buf_pos = 0;
        return buf_pos < buf_len ? buf[buf_pos++] : -1;
    };
    int c = get();
    while (c != '-' && (c < '0' || c > '9')) c = get();
    int x = 0;
    for (; c >= '0' && c <= '9'; c = get()) x = x * 10 + (c - '0');
    return x;
}
int main() {
    int n = rd();
    vector<vector<int>> adj(n);
    for (int i = 0; i + 1 < n; i++) {
        int u = rd() - 1, v = rd() - 1;
        adj[u].push_back(v), adj[v].push_back(u);
    }
    vector<int> par(n, -1), dep(n, 0), sz(n, 1), heavy(n, -1), top(n), tin(n), at(n), order;
    vector<vector<int>> kid(n);
    order.reserve(n);
    {
        vector<char> seen(n, 0);
        vector<pair<int, int>> st{{0, 0}};
        while (!st.empty()) {
            auto [u, p] = st.back();
            st.pop_back();
            if (seen[u]) continue;
            seen[u] = 1;
            if (u != p) par[u] = p, dep[u] = dep[p] + 1, kid[p].push_back(u);
            order.push_back(u);
            size_t base = st.size();
            for (int c : adj[u])
                if (!seen[c]) st.push_back({c, u});
            reverse(st.begin() + base, st.end());
        }
    }
    for (int i = n - 1; i > 0; i--) sz[par[order[i]]] += sz[order[i]];
    for (int u : order)
        for (int c : kid[u])
            if (heavy[u] < 0 || sz[c] > sz[heavy[u]]) heavy[u] = c;
    for (int v = 0; v < n; v++) top[v] = v;
    tin[0] = 0;
    for (int u : order) {
        int next = tin[u] + 1;
        if (heavy[u] >= 0) top[heavy[u]] = top[u], tin[heavy[u]] = next, next += sz[heavy[u]];
        for (int c : kid[u])
            if (c != heavy[u]) tin[c] = next, next += sz[c];
    }
    for (int v = 0; v < n; v++) at[tin[v]] = v;
    auto lca = [&](int a, int b) {
        while (top[a] != top[b]) {
            if (dep[top[a]] < dep[top[b]]) swap(a, b);
            a = par[top[a]];
        }
        return dep[a] < dep[b] ? a : b;
    };
    const ll NEG = LLONG_MIN;
    auto lift = [&](ll h) { return h + 1; };
    auto finish = [&](ll h) { return max(h, 0LL); };
    vector<ll> below(n), above(n, NEG), far(n);
    for (int p = n - 1; p >= 0; p--) {
        int v = at[p];
        ll acc = NEG;
        for (int c : kid[v]) acc = max(acc, lift(below[c]));
        below[v] = finish(acc);
    }
    vector<ll> parts, after;
    for (int p = 0; p < n; p++) {
        int v = at[p];
        auto &cs = kid[v];
        int k = cs.size();
        parts.assign(k, 0), after.assign(k + 1, NEG);
        for (int i = 0; i < k; i++) parts[i] = lift(below[cs[i]]);
        for (int i = k - 1; i >= 0; i--) after[i] = max(parts[i], after[i + 1]);
        ll before = above[v];
        for (int i = 0; i < k; i++) {
            above[cs[i]] = lift(finish(max(before, after[i + 1])));
            before = max(before, parts[i]);
        }
        far[v] = finish(before);
    }
    int q = rd();
    ll hash = 0;
    for (int i = 0; i < q; i++) {
        int a = rd() - 1, b = rd() - 1;
        int l = lca(a, b);
        hash = (hash * 31 + l + (dep[a] + dep[b] - 2 * dep[l])) % 1000000007;
    }
    vector<char> dead(n, 0);
    vector<int> csz(n), bpar(n, -1), from(n, -1), cpar(n, -1), cdep(n, 0);
    vector<vector<array<int, 3>>> anc(n);
    vector<vector<int>> held(n);
    vector<tuple<int, int, int>> todo{{0, -1, 0}};
    while (!todo.empty()) {
        auto [entry, up, d] = todo.back();
        todo.pop_back();
        vector<int> ord{entry};
        bpar[entry] = -1;
        for (size_t h = 0; h < ord.size(); h++) {
            int u = ord[h];
            for (int w : adj[u])
                if (!dead[w] && w != bpar[u] && w != entry) bpar[w] = u, ord.push_back(w);
        }
        for (int u : ord) csz[u] = 1;
        for (size_t i = ord.size() - 1; i >= 1; i--) csz[bpar[ord[i]]] += csz[ord[i]];
        int total = ord.size(), c = entry;
        for (bool moved = true; moved;) {
            moved = false;
            for (int w : adj[c])
                if (!dead[w] && w != bpar[c] && w != entry && 2 * csz[w] > total) {
                    c = w, moved = true;
                    break;
                }
        }
        cpar[c] = up, cdep[c] = d, dead[c] = 1;
        vector<array<int, 3>> qq{{c, 0, c}};
        from[c] = -1;
        for (size_t h = 0; h < qq.size(); h++) {
            auto [u, du, b] = qq[h];
            anc[u].push_back({c, du, b});
            held[c].push_back(u);
            for (int w : adj[u])
                if (!dead[w] && w != from[u]) from[w] = u, qq.push_back({w, du + 1, u == c ? w : b});
        }
        for (int w : adj[c])
            if (!dead[w]) todo.push_back({w, c, d + 1});
    }
    for (int v = 0; v < n; v++) reverse(anc[v].begin(), anc[v].end());
    ll spread = 0, total = 0;
    for (int v = 0; v < n; v++)
        for (auto &x : anc[v]) spread += x[1];
    for (int v = 0; v < n; v++) total += far[v];
    printf("%lld %lld %lld\n", total, hash, spread);
}
