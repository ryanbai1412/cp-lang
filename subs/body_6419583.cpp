
void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
struct R {
    mt19937_64 cowng;
    R() : R(chrono::steady_clock::now().time_since_epoch().count()) {}
    R(u64 seed) : cowng(seed) {}
    long rint(long l, long r) {
        return cowng() % (r - l + 1) + l;
    }
};
template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}
template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}
struct E {
    int u, v, r, p;
};
void solve() {
    int n, m;
    cin >> n >> m;
    E edges[m];
    for (auto& [u, v, r, p] : edges) {
        cin >> u >> v >> r >> p;
        u--;
        v--;
    }
    vector<int> graph[n], igraph[n];
    for (int i = 0; i < m; i++) {
        auto& [u, v, _r, _p] = edges[i];
        graph[u].push_back(i);
        igraph[v].push_back(i);
    }
    bool bad_edge[m] {};
    {
        vector<int> q;
        int deg[n] {};
        bool vis[n] {};
        for (int i = 0; i < n; i++) {
            deg[i] = sz(graph[i]);
        }
        auto push = [&](int u) -> void {
            if (deg[u]) {
                return;
            }
            assert(!vis[u]);
            vis[u] = true;
            q.push_back(u);
        };
        for (int i = 0; i < n; i++) {
            push(i);
        }
        while (sz(q)) {
            int u = q.back();
            q.pop_back();
            for (auto& ei : igraph[u]) {
                deg[edges[ei].u]--;
                push(edges[ei].u);
            }
        }
        for (int i = 0; i < m; i++) {
            bad_edge[i] = vis[edges[i].u] || vis[edges[i].v];
        }
    }
    for (auto& a : graph) {
        a.erase(remove_if(begin(a), end(a),
                          [&](int ei) -> bool {
            return bad_edge[ei];
        }),
                end(a));
    }
    for (auto& a : igraph) {
        a.erase(remove_if(begin(a), end(a),
                          [&](int ei) -> bool {
            return bad_edge[ei];
        }),
                end(a));
    }
    long arr_v[n];
    memset(arr_v, -1, sizeof(arr_v));
    auto edge_val = [&](int i) -> long {
        auto& [_u, v, r, p] = edges[i];
        return max(long(r), arr_v[v] - p);
    };
    priority_queue<pair<long, int>> pq;
    auto push = [&](int ei) -> void {
        if (bad_edge[ei]) {
            return;
        }
        pq.emplace(edge_val(ei), ei);
    };
    for (int i = 0; i < m; i++) {
        push(i);
    }
    bool e_vis[m] {};
    int deg[n] {};
    for (int i = 0; i < n; i++) {
        deg[i] = sz(graph[i]);
    }
    while (sz(pq)) {
        auto [d, ei] = pq.top();
        pq.pop();
        if (e_vis[ei]) {
            continue;
        }
        e_vis[ei] = true;
        assert(d == edge_val(ei));
        auto& [u, v, r, p] = edges[ei];
        if (--deg[u]) {
            continue;
        }
        arr_v[u] = d;
        for (auto& a : igraph[u]) {
            push(a);
        }
    }
    for (int i = 0; i < n; i++) {
        cout << arr_v[i] << " \n"[i == n - 1];
    }
}
