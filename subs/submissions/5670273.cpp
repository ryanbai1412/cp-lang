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

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

constexpr int maxn = 2e3 + 5;

void dfs(int u, vector<char>& vis, const vector<vector<int>>& graph) {
    if (vis[u]) {
        return;
    }
    vis[u] = true;
    for (auto& v : graph[u]) {
        dfs(v, vis, graph);
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int edges[m][2];
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }

    int dist[n];
    memcpy(dist, arr, sizeof(dist));
    for (int it = 0; it < n; it++) {
        for (auto& [u, v] : edges) {
            dist[u] = min(dist[u], dist[v] - 1);
        }
    }

    {
        int ord[n];
        iota(ord, ord + n, 0);
        sort(ord, ord + n, CmpByKey([&](int u) -> int { return dist[u]; }));

        for (int i = 0; i < n; i++) {
            cout << ord[i] + 1 << " \n"[i == n - 1];
        }
    }

    vector<vector<int>> igraph(n);
    for (auto& [u, v] : edges) {
        igraph[v].push_back(u);
    }

    for (int u = 0; u < n; u++) {
        vector<char> vis(n);
        dfs(u, vis, igraph);

        int cnt[n] {};
        for (int i = 0; i < n; i++) {
            if (!vis[i]) {
                cnt[dist[i]]++;
            }
        }

        bool filled[n] {};
        for (int i = n - 1, c = 0; i >= 0; i--) {
            c += cnt[i];
            if (c) {
                filled[i] = true;
                c--;
            }
        }

        int ans = -1;
        for (int i = 0; i < n; i++) {
            if (!filled[i]) {
                ans = i;
            }
        }
        assert(ans != -1);
        cout << ans + 1 << " \n"[u == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}