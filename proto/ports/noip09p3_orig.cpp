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

constexpr int maxn = 1e5 + 5;

bool vis[maxn];
vector<int> graph[maxn];

void dfs(int u) {
    if (vis[u]) {
        return;
    }
    vis[u] = true;
    for (auto& v : graph[u]) {
        dfs(v);
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    for (int i = 0; i < m; i++) {
        int u, v, t;
        cin >> u >> v >> t;
        u--;
        v--;
        if (t == 1) {
            graph[v].push_back(u);
        } else {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
    }
    dfs(n - 1);
    int dist[n][2];
    memset(dist, 0x3f, sizeof(dist));
    min_pq<tuple<int, int, int>> pq;
    auto push = [&](int u, int t, int d) -> void {
        if (d < dist[u][t]) {
            dist[u][t] = d;
            pq.emplace(d, u, t);
        }
    };
    for (int i = 0; i < n; i++) {
        if (vis[i]) {
            push(i, 0, -arr[i]);
        }
    }
    while (sz(pq)) {
        auto [d, u, t] = pq.top();
        pq.pop();
        if (d > dist[u][t]) {
            continue;
        }
        assert(d == dist[u][t]);
        if (t == 1 && u == 0) {
            cout << -d << endl;
            return;
        }
        for (auto& v : graph[u]) {
            push(v, t, d);
        }
        if (t == 0) {
            push(u, 1, d + arr[u]);
        }
    }
    cout << 0 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}