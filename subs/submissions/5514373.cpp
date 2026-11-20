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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

template <typename T>
struct ST {
    int n;
    vector<vector<T>> v;

    ST(const vector<T>& arr) : n(sz(arr)), v(__lg(n) + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= __lg(n); i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = min(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    T query(int l, int r) const {
        int x = __lg(r - l);
        return min(v[x][l], v[x][r - (1 << x)]);
    }
};

constexpr int maxn = 5e4 + 5;

int tin[maxn], tout[maxn], depth[maxn];
vector<int> ord, trie[maxn];

void dfs1(int u, int p, int d) {
    tin[u] = sz(ord);
    depth[u] = d;
    ord.push_back(u);
    for (auto& v : trie[u]) {
        if (v == p) {
            continue;
        }
        dfs1(v, u, d + 1);
        ord.push_back(u);
    }
    tout[u] = sz(ord);
}

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    ord.clear();
    for (int i = 0; i < kv; i++) {
        trie[i].clear();
    }
    int edges[m][4];
    for (auto& [u, v, w, t] : edges) {
        cin >> u >> v >> w >> t;
        u--;
        v--;
        t--;
    }
    vector<int> igraph[n], graph[n];
    for (int i = 0; i < m; i++) {
        graph[edges[i][0]].push_back(i);
        igraph[edges[i][1]].push_back(i);
    }
    for (int i = 0; i < kv - 1; i++) {
        int u, v, _w;
        cin >> u >> v >> _w;
        u--;
        v--;
        trie[u].push_back(v);
        trie[v].push_back(u);
    }
    dfs1(0, -1, 0);
    vector<pair<int, int>> stv;
    for (auto& a : ord) {
        stv.emplace_back(tin[a], a);
    }
    ST st(stv);
    auto lca = [&](int u, int v) -> int {
        if (tin[u] > tin[v]) {
            swap(u, v);
        }
        return st.query(tin[u], tout[v]).second;
    };
    int rep[m];
    memset(rep, -1, sizeof(rep));
    for (int i = 0; i < n; i++) {
        map<int, int> mp;
        for (auto& a : igraph[i]) {
            auto [it, inserted] = mp.emplace(edges[a][3], a);
            if (!inserted) {
                rep[a] = it->second;
            }
        }
    }
    long dist[2 * m];
    memset(dist, 0x3f, sizeof(dist));
    min_pq<pair<long, int>> pq;
    auto push = [&](int u, long d) -> void {
        if (d >= dist[u]) {
            return;
        }
        dist[u] = d;
        pq.emplace(d, u);
    };
    for (auto& a : graph[0]) {
        push(a, 0);
    }
    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (dist[u] != d) {
            continue;
        }
        if (u < m) {
            push(u + m, d + edges[u][2]);
            continue;
        }
        u -= m;
        if (rep[u] != -1) {
            push(rep[u] + m, d);
            continue;
        }
        for (auto& v : graph[edges[u][1]]) {
            push(v, d + depth[lca(edges[u][3], edges[v][3])]);
        }
    }
    long ans[n];
    memset(ans, 0x3f, sizeof(ans));
    for (int i = 0; i < m; i++) {
        ans[edges[i][1]] = min(ans[edges[i][1]], dist[i] + edges[i][2]);
    }
    for (int i = 1; i < n; i++) {
        cout << ans[i] << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}