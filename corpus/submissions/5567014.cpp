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
            int cnt = 0;
            for (auto& a : cnodes) {
                int ccnt = 0;
                for (auto& b : graph[a]) {
                    ccnt += !inode[b];
                }
                cnt += ccnt > 0;
            }
            ans |= cnt >= 2;
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