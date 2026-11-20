#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct DSU {
    vector<int> p;

    DSU(int n) : p(n, -1) {}

    int find(int u) {
        return p[u] < 0 ? u : (p[u] = find(p[u]));
    }

    bool merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return false;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[v] += p[u];
        p[u] = v;
        return true;
    }
};

namespace s1 {

constexpr int maxn = 1e3 + 5;

int n, m, cc, tin[maxn], tout[maxn];
bool ans, vis[maxn];
vector<pair<int, int>> edges;
vector<int> st, graph[maxn];

void dfs(int u, int p) {
    st.push_back(u);
    tin[u] = cc++;
    vis[u] = true;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        } else if (vis[v]) {
            if (tin[u] <= tin[v]) {
                continue;
            }
            vector<int> cnodes(find(begin(st), end(st), v), st.end());
            dbg("A", sz(cnodes), u, v);
            bool inode[n] {};
            for (auto& a : cnodes) {
                dbg(a);
                inode[a] = true;
            }
            DSU dsu(n);
            for (auto& [u, v] : edges) {
                if (inode[u] && inode[v]) {
                    continue;
                }
                dsu.merge(u, v);
            }
            bool comps[n] {};
            for (auto& a : cnodes) {
                int cu = dsu.find(a);
                if (comps[cu]) {
                    ans = true;
                }
                comps[cu] = true;
            }
            continue;
        }
        dfs(v, u);
    }
    tout[u] = cc;
    st.pop_back();
}

void solve() {
    cin >> n >> m;
    cc = 0;
    ans = false;
    for (int i = 0; i < n; i++) {
        vis[i] = false;
        graph[i].clear();
    }
    edges.resize(m);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        dfs(i, -1);
    }
    if (ans) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
    }
}

}  // namespace s1

namespace s2 {

constexpr int maxn = 1e3 + 5;

bool has_cyc, vis[maxn];
vector<int> graph[maxn];

void dfs(int u, int p) {
    assert(!vis[u]);
    vis[u] = true;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        } else if (vis[v]) {
            has_cyc = true;
            continue;
        }
        dfs(v, u);
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        vis[i] = false;
        graph[i].clear();
    }
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    has_cyc = false;
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        dfs(i, -1);
    }
    if (has_cyc) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
    }
}

}  // namespace s2

namespace s3 {

constexpr int maxn = 1e5 + 5;

int cc, tin[maxn], tout[maxn], depth[maxn], par[maxn];
bool ans, up[maxn], vis[maxn];
vector<int> graph[maxn];

void dfs(int u, int p, int d) {
    tin[u] = cc++;
    vis[u] = true;
    depth[u] = d;
    par[u] = p;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        } else if (vis[v]) {
            if (tin[u] <= tin[v]) {
                continue;
            }
            up[u] = true;
            ans |= depth[u] - depth[v] >= 3;
            continue;
        }
        dfs(v, u, d + 1);
    }
    tout[u] = cc;
}

void solve() {
    int n, m;
    cin >> n >> m;
    cc = 0;
    ans = false;
    for (int i = 0; i < n; i++) {
        vis[i] = false;
        up[i] = false;
        graph[i].clear();
    }
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            continue;
        }
        dfs(i, -1, 0);
    }
    for (int i = 0; i < n; i++) {
        int cnt = 0;
        for (auto& v : graph[i]) {
            if (par[v] != i) {
                continue;
            }
            cnt += up[v];
        }
        ans |= cnt >= 2;
    }
    for (int i = 0; i < n; i++) {
        ans |= up[i] && par[i] != -1 && up[par[i]];
    }
    if (ans) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
    }
}

}  // namespace s3

namespace s4 {

void solve() {
    int n, m;
    cin >> n >> m;
    vector<int> graph[n];
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    for (auto& a : graph) {
        if (sz(a) >= 3) {
            cout << "YES" << endl;
            return;
        }
    }
    cout << "NO" << endl;
}

}  // namespace s4

namespace s5 {

constexpr int maxn = 1e3 + 5;

int n, m, cc, tin[maxn], tout[maxn], low[maxn], par[maxn];
bool ans, vis[maxn], bgraph[maxn][maxn], artic[maxn], vis2[maxn];
vector<pair<int, int>> edges;
vector<int> ord2, st, graph[maxn];
vector<vector<int>> bccs;
// laziness
set<pair<int, int>> bridges;

void dfs2(int u) {
    if (vis2[u]) {
        return;
    }
    vis2[u] = true;
    ord2.push_back(u);
    for (auto& v : graph[u]) {
        if (bridges.count({u, v}) || bridges.count({v, u})) {
            continue;
        }
        dfs2(v);
    }
}

void dfs3(int u) {
    ord2.push_back(u);
    for (auto& v : graph[u]) {
        if (par[v] != u || tin[u] <= low[v]) {
            continue;
        }
        dfs3(v);
    }
}

void gen_bccs() {
    // bccs.clear();
    // fill(vis2, vis2 + n, false);
    // for (int i = 0; i < n; i++) {
    //     if (vis2[i]) {
    //         continue;
    //     }
    //     ord2.clear();
    //     dfs2(i);
    //     bccs.push_back(ord2);
    // }
    bccs.clear();
    for (int i = 0; i < n; i++) {
        if (i && !artic[i]) {
            continue;
        }
        for (auto& v : graph[i]) {
            if (par[v] != i || low[v] < tin[i]) {
                continue;
            }
            ord2.clear();
            dfs3(v);
            ord2.push_back(i);
            bccs.push_back(ord2);
        }
    }
}

bool check5(array<int, 5> nodes) {
    sort(nodes.begin() + 1, nodes.begin() + 5);
    do {
        bool is_cyc = bgraph[nodes[0]][nodes[4]];
        for (int i = 0; i < 4; i++) {
            is_cyc &= bgraph[nodes[i]][nodes[i + 1]];
        }
        int out_cnt = 0;
        for (auto& a : nodes) {
            int csz = sz(graph[a]);
            for (auto& b : nodes) {
                csz -= bgraph[a][b];
            }
            out_cnt += csz > 0;
        }
        if (is_cyc && out_cnt >= 2) {
            dbg(out_cnt);
            return true;
        }
    } while (next_permutation(nodes.begin() + 1, nodes.begin() + 5));
    return false;
}

void process_bcc(const vector<int>& nodes) {
    if (sz(nodes) < 5) {
        return;
    } else if (sz(nodes) == 5) {
        ans |= check5(({
            array<int, 5> v;
            copy(begin(nodes), end(nodes), v.begin());
            v;
        }));
        return;
    }
    int out_cnt = 0;
    for (auto& a : nodes) {
        out_cnt += sz(graph[a]) > 2;
    }
    ans |= out_cnt >= 2;
}

void dfs(int u, int p) {
    dbg(u, p);
    st.push_back(u);
    par[u] = p;
    low[u] = tin[u] = cc++;
    vis[u] = true;
    int ch = 0;
    bool is_artic = false;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        } else if (vis[v]) {
            low[u] = min(low[u], tin[v]);
            continue;
        }
        ch++;
        dfs(v, u);
        low[u] = min(low[u], low[v]);
        is_artic |= tin[u] <= low[v];
        if (tin[u] < low[v]) {
            dbg("BRIDGE", u, v);
            bridges.emplace(u, v);
        }
    }
    if (p == -1) {
        artic[u] = ch > 1;
    } else {
        artic[u] = is_artic;
    }
    tout[u] = cc;
    st.pop_back();
}

void solve() {
    cin >> n >> m;
    cc = 0;
    ans = false;
    bridges.clear();
    for (int i = 0; i < n; i++) {
        vis[i] = artic[i] = false;
        graph[i].clear();
        fill(bgraph[i], bgraph[i] + n, false);
    }
    edges.resize(m);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
        dbg(u, v);
        bgraph[u][v] = bgraph[v][u] = true;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    auto check3 = [&](int a, int b, int c) -> bool {
        int cnt =
            (sz(graph[a]) >= 3) + (sz(graph[b]) >= 3) + (sz(graph[c]) >= 3);
        int deg_max = 0;
        for (auto& u : {a, b, c}) {
            deg_max = max(deg_max, sz(graph[u]) - 2);
        }
        if (cnt >= 2 && deg_max == 1) {
            set<int> vis;
            for (auto& u : {a, b, c}) {
                vis.insert(begin(graph[u]), end(graph[u]));
            }
            for (auto& u : {a, b, c}) {
                vis.erase(u);
            }
            if (sz(vis) <= 1) {
                cnt = 0;
            }
        }
        return bgraph[a][b] && bgraph[b][c] && bgraph[a][c] && cnt >= 2;
    };
    auto check4 = [&](int a, int b, int c, int d) -> bool {
        bool is_cyc =
            bgraph[a][b] && bgraph[b][c] && bgraph[c][d] && bgraph[d][a];
        int out_nodes = 0, deg_max = 0;
        for (auto& u : {a, b, c, d}) {
            int csz = sz(graph[u]);
            for (auto& v : {a, b, c, d}) {
                csz -= bgraph[u][v];
            }
            deg_max = max(deg_max, csz);
            out_nodes += csz > 0;
        }
        if (out_nodes >= 2 && deg_max == 1) {
            set<int> vis;
            for (auto& u : {a, b, c, d}) {
                vis.insert(begin(graph[u]), end(graph[u]));
            }
            for (auto& u : {a, b, c, d}) {
                vis.erase(u);
            }
            if (sz(vis) <= 1) {
                out_nodes = 0;
            }
        }
        dbg(is_cyc && out_nodes >= 2);
        return is_cyc && out_nodes >= 2;
    };
    for (int i = 0; i < sz(edges); i++) {
        for (int j = i + 1; j < sz(edges); j++) {
            auto& [u1, v1] = edges[i];
            auto& [u2, v2] = edges[j];
            vector<int> cnodes {u1, v1, u2, v2};
            sort(begin(cnodes), end(cnodes));
            cnodes.erase(unique(begin(cnodes), end(cnodes)), cnodes.end());
            if (sz(cnodes) == 3) {
                ans |= check3(cnodes[0], cnodes[1], cnodes[2]);
                continue;
            }
            ans |= check4(u1, v1, u2, v2) || check4(u1, v1, v2, u2);
        }
    }
    dfs(0, -1);
    for (int i = 0; i < n; i++) {
        if (artic[i]) {
            dbg(i);
        }
    }
    gen_bccs();
    for (auto& a : bccs) {
        cerr << "BCC:";
        for (auto& b : a) {
            cerr << " " << b;
        }
        cerr << endl;
        process_bcc(a);
    }
    if (ans) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
    }
}

}  // namespace s5

void solve() {
    int c, tcs;
    cin >> c >> tcs;
    while (tcs--) {
        if (c == 1) {
            s1::solve();
        } else if (c == 2) {
            s2::solve();
        } else if (c == 3) {
            s3::solve();
        } else if (c == 4) {
            s4::solve();
        } else if (c == 5) {
            s5::solve();
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}