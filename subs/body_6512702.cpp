
template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<>>;
template <typename Cb>
struct CmpByKey {
    Cb cb;
    CmpByKey(Cb cb) : cb(cb) {}
    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};
vector<vector<pair<int, int>>> to_adj_list(int n,
                                           const vector<array<int, 3>>& edges) {
    vector<vector<pair<int, int>>> graph(n);
    for (auto& [u, v, w] : edges) {
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    return graph;
}
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
    void set_par(int u, int v) {
        u = find(u);
        v = find(v);
        assert(u != v);
        p[v] += p[u];
        p[u] = v;
    }
};
vector<long> dijkstra(const vector<vector<pair<int, int>>>& graph, int src) {
    int n = sz(graph);
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
struct BLifter {
    int n, logn;
    vector<vector<int>> lift;
    BLifter(const vector<int>& arr)
        : n(sz(arr)), logn(__lg(n + 10) + 1), lift(logn, vector(n, 0)) {
        lift[0] = arr;
        for (int i = 1; i < logn; i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }
    template <typename Cb>
    int bsearch(int u, const Cb& cb) {
        for (int i = logn - 1; i >= 0; i--) {
            int nu = lift[i][u];
            if (nu != -1 && cb(nu)) {
                u = nu;
            }
        }
        return u;
    }
};
void solve() {
    int n, m;
    cin >> n >> m;
    vector<array<int, 3>> edges_water(m), edges_weight(m);
    for (int i = 0; i < m; i++) {
        int u, v, w, a;
        cin >> u >> v >> w >> a;
        u--;
        v--;
        edges_water[i] = {u, v, a};
        edges_weight[i] = {u, v, w};
    }
    auto dist = dijkstra(to_adj_list(n, edges_weight), 0);
    DSU dsu(2 * n);
    int id = n;
    vector lift(2 * n, -1), n_water(2 * n, int(1e9 + 5));
    vector<long> n_dist(2 * n, -1);
    copy(begin(dist), end(dist), begin(n_dist));
    sort(begin(edges_water), end(edges_water),
         CmpByKey([&](const auto& a) -> int {
        return -a[2];
    }));
    for (auto& [u, v, w] : edges_water) {
        int cu = dsu.find(u), cv = dsu.find(v);
        if (cu == cv) {
            continue;
        }
        int nx = id++;
        dsu.set_par(cu, nx);
        dsu.set_par(cv, nx);
        lift[cu] = lift[cv] = nx;
        n_water[nx] = w;
        n_dist[nx] = min(n_dist[cu], n_dist[cv]);
    }
    BLifter blift(lift);
    int q, qk, qs, lastans = 0;
    cin >> q >> qk >> qs;
    while (q--) {
        int u, p;
        cin >> u >> p;
        u = (u + qk * lastans - 1 + n) % n;
        p = (p + qk * lastans) % (qs + 1);
        int nu = blift.bsearch(u, [&](int v) -> bool {
            return p < n_water[v];
        });
        cout << (lastans = n_dist[nu]) << endl;
    }
}
