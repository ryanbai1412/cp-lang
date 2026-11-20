#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
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
    
    bool conn(int u, int v) {
        return find(u) == find(v);
    }
};

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    array<int, 4> edges[m];
    for (auto& [w, _, u, v] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
    }
    for (int i = 0; i < m; i++) {
        edges[i][1] = i >= n - 1;
    }
    sort(edges, edges + m);
    bool vis[m] {};
    int mx = 0;
    {
        DSU dsu(n);
        for (int i = 0; i < m; i++) {
            auto& [w, _b, u, v] = edges[i];
            if (dsu.merge(u, v)) {
                vis[i] = true;
                mx = max(mx, w);
            }
        }
    }
    DSU dsu(n);
    for (int i = 0; i < m; i++) {
        auto& [w, b, u, v] = edges[i];
        if (vis[i] && (!b || w < mx)) {
            dsu.merge(u, v);
        }
    }
    bool flag = false;
    if (mx <= kv) {
        for (int i = 0; i < m; i++) {
            auto& [w, b, u, v] = edges[i];
            if (!vis[i] && !b && !dsu.conn(u, v) && w <= kv) {
                flag = true;
            }
        }
    }
    dbg(flag, mx, kv);
    int cans = -flag;
    for (int i = 0; i < m; i++) {
        if (vis[i]) {
            cans += edges[i][1];
        }
    }
    cout << cans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}