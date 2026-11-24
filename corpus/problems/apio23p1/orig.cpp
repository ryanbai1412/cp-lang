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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

vector<double> dijkstra(const vector<vector<pair<int, double>>>& graph,
                        const vector<pair<int, double>>& srcs) {
    int n = sz(graph);
    vector<double> dist(n, 1e18);
    min_pq<pair<double, int>> pq;
    auto push = [&](int u, double d) -> void {
        if (d >= dist[u] - 1e-9) {
            return;
        }
        dist[u] = d;
        pq.emplace(d, u);
    };
    for (auto& [u, d] : srcs) {
        push(u, d);
    }
    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (d != dist[u]) {
            continue;
        }
        for (auto& [v, w] : graph[u]) {
            push(v, d + w);
        }
    }
    return dist;
}

double solve(int n,
             int m,
             int kv,
             int dest,
             vector<int> edgesu,
             vector<int> edgesv,
             vector<int> edgesw,
             vector<int> arr) {
    kv = min(kv, 80);
    vector<tuple<int, int, double>> edges;
    for (int i = 0; i < m; i++) {
        edges.emplace_back(edgesu[i], edgesv[i], edgesw[i]);
    }
    vector<vector<pair<int, double>>> graph(n);
    for (auto& [u, v, w] : edges) {
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    graph[dest].clear();
    auto dist = dijkstra(graph, {{0, 0}});
    if (dist[dest] >= 1e18) {
        return -1;
    }
    // if (*max_element(begin(arr), end(arr)) == 1) {
    //     dist = dijkstra(graph, {{dest, 0}});
    //     double ans = 1e18;
    //     for (int i = 0; i < n; i++) {
    //         if (!i || !arr[i]) {
    //             ans = min(ans, dist[i]);
    //         }
    //     }
    //     return ans;
    // }
    dbg(dist[0], dist[1], dist[2]);
    {
        vector<pair<int, double>> srcs;
        srcs.emplace_back(0, 0);
        for (int i = 0; i < n; i++) {
            if (dist[i] < 1e18 && !arr[i]) {
                srcs.emplace_back(i, 0);
            }
        }
        dist = dijkstra(graph, srcs);
    }
    dbg(dist[0], dist[1], dist[2]);
    for (int it = 0; it < kv; it++) {
        vector<pair<int, double>> srcs;
        for (int i = 0; i < n; i++) {
            srcs.emplace_back(i, dist[i]);
            if (dist[i] < 1e18 && arr[i] == 2) {
                for (auto& [v, w] : graph[i]) {
                    srcs.emplace_back(v, dist[i] / 2 + w);
                }
            }
        }
        dist = dijkstra(graph, srcs);
    }
    return dist[dest];
}

// The sample grader, reading the input the port reads (corpus/submissions/5593318.cpp is the grader function).
int main() {
    int t; scanf("%d", &t);
    while (t--) {
        int n, m, k, h; scanf("%d %d %d %d", &n, &m, &k, &h);
        vector<int> arr(n), u(m), v(m), c(m);
        for (auto& x : arr) scanf("%d", &x);
        for (int i = 0; i < m; i++) scanf("%d %d %d", &u[i], &v[i], &c[i]);
        double r = solve(n, m, k, h, u, v, c, arr);
        if (r < -0.5) puts("-1"); else printf("%.6f\n", r);
    }
}
