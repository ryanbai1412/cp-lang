
struct Solver {
    int n;
    vector<char> vis, has_par;
    set<int> to_vis;
    vector<pair<int, int>> dfs_tree;
    vector<vector<int>> graph;
    Solver(int n, const vector<pair<int, int>>& edges)
        : n(n), vis(n), has_par(n), graph(n) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
        }
        for (int i = 0; i < n; i++) {
            to_vis.insert(i);
        }
        for (int i = 0; i < n; i++) {
            if (to_vis.count(i)) {
                to_vis.erase(i);
                dfs(i, -1);
                to_vis.insert(i);
            }
        }
    }
    void dfs(int u, int p) {
        assert(!has_par[u]);
        if (p != -1) {
            has_par[u] = true;
            dfs_tree.emplace_back(p, u);
        }
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        set<int> can_vis, cant_vis;
        swap(can_vis, to_vis);
        for (auto& v : graph[u]) {
            auto it = can_vis.find(v);
            if (it == can_vis.end()) {
                continue;
            }
            cant_vis.insert(v);
            can_vis.erase(it);
        }
        swap(cant_vis, to_vis);
        for (auto& v : can_vis) {
            dfs(v, u);
        }
    }
};
struct Checker {
    int n;
    vector<char> vis;
    vector<vector<int>> graph;
    Checker(const vector<pair<int, int>>& edges) : n(sz(edges) + 1),  vis(n), graph(n) {
        int ideg[n] {};
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            ideg[v]++;
        }
        int zdeg = 0;
        for (int i = 0; i < n; i++) {
            zdeg += !ideg[i];
            if (!ideg[i]) {
                dfs(i);
            }
        }
        assert(zdeg == 1);
        assert(vis == vector<char>(n, true));
    }
    void dfs(int u) {
        assert(!vis[u]);
        vis[u] = true;
        for (auto& v : graph[u]) {
            dfs(v);
        }
    }
};
constexpr int maxn = 3e5 + 5;
bool has_cyc, vis[maxn];
vector<int> ord, graph[maxn];
void dfs(int u) {
    if (vis[u]) {
        has_cyc = true;
        return;
    }
    vis[u] = true;
    for (auto& v : graph[u]) {
        dfs(v);
    }
    ord.push_back(u);
}
void solve() {
    int n, m1, m2;
    cin >> n >> m1 >> m2;
    vector<pair<int, int>> edges1(m1), edges2(m2);
    for (auto& [u, v] : edges1) {
        cin >> u >> v;
        swap(u, v);
    }
    for (auto& [u, v] : edges2) {
        cin >> u >> v;
        swap(u, v);
    }
    int ideg[n] {};
    for (auto& [u, v] : edges1) {
        graph[u].push_back(v);
        ideg[v]++;
    }
    int comp_id = 0, comp[n], rep[n];
    vector<int> comps[n];
    for (int i = 0; i < n; i++) {
        if (ideg[i]) {
            continue;
        }
        ord.clear();
        dfs(i);
        rep[comp_id] = i;
        comps[comp_id] = ord;
        for (auto& a : ord) {
            comp[a] = comp_id;
        }
        comp_id++;
    }
    if (has_cyc) {
        cout << "NO" << endl;
        return;
    }
    for (int i = 0; i < n; i++) {
        if (!vis[i]) {
            cout << "NO" << endl;
            return;
        }
    }
    set<int> igraph2[n];
    for (auto& [u, v] : edges2) {
        igraph2[v].insert(u);
    }
    vector<pair<int, int>> cc_e;
    for (int i = 0; i < comp_id; i++) {
        map<int, int> adj_comps;
        for (auto& v : igraph2[rep[i]]) {
            adj_comps[comp[v]]++;
        }
        for (auto& [k, v] : adj_comps) {
            if (v == sz(comps[k])) {
                cc_e.emplace_back(k, i);
            }
        }
    }
    auto comp_ans = Solver(comp_id, cc_e).dfs_tree;
    if (sz(comp_ans) != comp_id - 1) {
        cout << "NO" << endl;
        return;
    }
    vector<pair<int, int>> ans = edges1;
    for (auto& [u, v] : comp_ans) {
        dbg(u, v);
        for (auto& a : comps[u]) {
            if (!igraph2[rep[v]].count(a)) {
                ans.emplace_back(a, rep[v]);
                break;
            }
        }
    }
    for (auto& [u, v] : ans) {
        cout << v << " " << u << endl;
    }
    cout.flush();
    assert(sz(ans) == n - 1);
    {
        set<pair<int, int>> s_ans(begin(ans), end(ans));
        for (auto& e : edges2) {
            assert(!s_ans.count(e));
        }
    }
    Checker _(ans);
}
