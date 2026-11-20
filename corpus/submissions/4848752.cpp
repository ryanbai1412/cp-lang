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

struct E {
    int u, v, r, p;
};

void solve() {
    int n, m;
    cin >> n >> m;
    vector<E> edges(m);
    for (auto& [u, v, r, p] : edges) {
        cin >> u >> v >> r >> p;
        u--;
        v--;
    }
    bool bad[n] {};
    {
        int deg[n] {};
        vector<int> graph[n];
        for (auto& [u, v, _r, _p] : edges) {
            deg[u]++;
            graph[v].push_back(u);
        }
        vector<int> q;
        auto push = [&](int u) -> void {
            if (!deg[u] && !bad[u]) {
                bad[u] = true;
                q.push_back(u);
            }
        };
        for (int i = 0; i < n; i++) {
            push(i);
        }
        while (sz(q)) {
            int u = q.back();
            q.pop_back();
            for (auto& v : graph[u]) {
                deg[v]--;
                push(v);
            }
        }
        vector<E> tmp;
        for (auto& a : edges) {
            if (!bad[a.u] && !bad[a.v]) {
                tmp.push_back(a);
            }
        }
        edges = std::move(tmp);
        m = sz(edges);
    }
    int ans[n], deg[n] {};
    memset(ans, -1, sizeof(ans));
    vector<int> graph[n];
    priority_queue<pair<int, int>> pq;
    for (int i = 0; i < m; i++) {
        deg[edges[i].u]++;
        graph[edges[i].v].push_back(i);
        pq.emplace(edges[i].r, i);
    }
    bool vis[m] {};
    while (sz(pq)) {
        auto [d, i] = pq.top();
        pq.pop();
        if (vis[i]) {
            continue;
        }
        vis[i] = true;
        auto& e = edges[i];
        if (!--deg[e.u]) {
            ans[e.u] = d;
            for (auto& j : graph[e.u]) {
                pq.emplace(max(edges[j].r, d - edges[j].p), j);
            }
        }
    }
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}