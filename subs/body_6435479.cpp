
void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<>>;
void solve() {
    int n, m;
    cin >> n >> m;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    vector<pair<int, int>> graph[n];
    for (int i = 0; i < m; i++) {
        int u, v, w;
        cin >> u >> v >> w;
        u--;
        v--;
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
    for (int i = 0; i < n; i++) {
        push(i, -arr[i]);
    }
    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (dist[u] != d) {
            continue;
        }
        for (auto& [v, w] : graph[u]) {
            push(v, d + w);
        }
    }
    for (auto& a : dist) {
        cout << -a << endl;
    }
}
