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
        p[v] += exchange(p[u], v);
        return true;
    }
};

vector<long> sp(int n, const vector<array<int, 3>>& edges, int src) {
    vector<pair<int, int>> graph[n];
    for (auto& [u, v, w] : edges) {
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    vector<long> dist(n, 1e18);
    min_pq<pair<long, int>> pq;
    auto push = [&](int u, long d) -> void {
        if (d >= dist[u]) {
            return;
        }
        dist[u] = d;
        pq.emplace(d, u);
    };
    push(src, 0);
    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        for (auto& [v, w] : graph[u]) {
            push(v, d + w);
        }
    }
    return dist;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto mp3 = [&](const vector<array<int, 4>>& arr) -> vector<array<int, 3>> {
        vector<array<int, 3>> ans;
        for (auto& [u, v, w, _c] : arr) {
            ans.push_back({u, v, w});
        }
        return ans;
    };
    vector<array<int, 4>> edges(m);
    for (auto& [u, v, w, c] : edges) {
        cin >> u >> v >> w >> c;
        u--;
        v--;
    }
    sort(begin(edges), end(edges), [&](const auto& a, const auto& b) -> bool {
        if (a[2] == b[2]) {
            return a[3] < b[3];
        }
        return a[2] < b[2];
    });
    vector<array<int, 3>> aedges;
    long ans = 0;
    for (auto& [u, v, w, c] : edges) {
        if (w < sp(n, aedges, u)[v]) {
            ans += c;
            aedges.push_back({u, v, w});
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}