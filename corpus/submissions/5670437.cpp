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

struct Flow {
    struct Edge {
        const int to;
        int cap;
    };

    int n;
    vector<Edge> edges;
    vector<basic_string<int>> graph;

    Flow(int n) : n(n), graph(n), vis(n), dist(n) {}

    void add_edge(int u, int v, int cap, int rcap) {
        assert(cap <= int(1e6) && rcap <= int(1e6));
        graph[u].push_back(sz(edges));
        edges.push_back({v, cap});
        graph[v].push_back(sz(edges));
        edges.push_back({u, rcap});
    }

    int ans, sink, min_cap;
    vector<char> vis;
    vector<int> st, dist;

    void dfs(int u) {
        if (vis[sink] || vis[u]) {
            return;
        }
        vis[u] = true;

        if (u == sink) {
            int c_cap = 1e9;
            for (auto& a : st) {
                c_cap = min(c_cap, edges[a].cap);
            }

            ans += c_cap;

            for (auto& a : st) {
                edges[a].cap -= c_cap;
                edges[a ^ 1].cap += c_cap;
            }

            return;
        }

        for (auto& ei : graph[u]) {
            auto& [v, cap] = edges[ei];

            if (dist[u] + 1 == dist[v] && cap >= min_cap) {
                st.push_back(ei);
                dfs(v);
                st.pop_back();
            }
        }
    }

    void bfs(int src) {
        fill(begin(dist), end(dist), -1);

        queue<int> q;
        auto push = [&](int u, int d) -> void {
            if (dist[u] != -1) {
                return;
            }
            dist[u] = d;
            q.push(u);
        };

        push(src, 0);

        while (sz(q)) {
            int u = q.front();
            q.pop();

            for (auto& ei : graph[u]) {
                auto& [v, cap] = edges[ei];
                if (cap) {
                    push(v, dist[u] + 1);
                }
            }
        }
    }

    int solve(int src, int _sink) {
        ans = 0;
        sink = _sink;

        while (true) {
            bfs(src);
            if (dist[sink] == -1) {
                break;
            }

            min_cap = 1 << 21;
            while (min_cap) {
                fill(begin(vis), end(vis), false);
                dfs(src);
                if (!vis[sink]) {
                    min_cap >>= 1;
                }
            }
        }

        return ans;
    }
};

void solve() {
    int n;
    cin >> n;
    n++;

    Flow flow(n * n);
    auto id = [&](int x, int y) -> int { return x * n + y; };

    int hori[n][n - 1][2], vert[n - 1][n][2];
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - 1; j++) {
            cin >> hori[i][j][0];
        }
    }
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n; j++) {
            cin >> vert[i][j][0];
        }
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - 1; j++) {
            cin >> hori[i][j][1];
        }
    }
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n; j++) {
            cin >> vert[i][j][1];
        }
    }

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - 1; j++) {
            auto [cap, rcap] = hori[i][j];
            flow.add_edge(id(i, j), id(i, j + 1), cap, rcap);
        }
    }
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n; j++) {
            auto [cap, rcap] = vert[i][j];
            flow.add_edge(id(i, j), id(i + 1, j), cap, rcap);
        }
    }

    cout << flow.solve(id(0, 0), id(n - 1, n - 1)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}