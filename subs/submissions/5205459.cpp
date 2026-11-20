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

constexpr int maxn = 1e4 + 5;

bool vis1[maxn];
vector<int> graph[maxn], igraph[maxn];

void dfs1(int u) {
    if (vis1[u]) {
        return;
    }
    vis1[u] = true;
    for (auto& v : igraph[u]) {
        dfs1(v);
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        igraph[v].push_back(u);
    }
    int src, dest;
    cin >> src >> dest;
    src--;
    dest--;
    dfs1(dest);
    bool good[n];
    memset(good, 1, sizeof(good));
    for (int i = 0; i < n; i++) {
        for (auto& v : graph[i]) {
            good[i] &= vis1[v];
        }
    }
    queue<int> q;
    int dist[maxn];
    memset(dist, -1, sizeof(dist));
    auto push = [&](int u, int d) -> void {
        if (dist[u] != -1 || !good[u]) {
            return;
        }
        dist[u] = d;
        q.push(u);
    };
    push(src, 0);
    while (sz(q)) {
        int u = q.front();
        q.pop();
        if (u == dest) {
            cout << dist[u] << endl;
            return;
        }
        for (auto& v : graph[u]) {
            push(v, dist[u] + 1);
        }
    }
    cout << -1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}