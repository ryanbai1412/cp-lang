
constexpr int maxn = 1e6 + 5;
bool cyc[maxn];
vector<int> st, graph[maxn];
int vis[maxn], depth[maxn], r_par[maxn], ans[maxn], ans_add;
vector<vector<int>> cycs;
void dfs(int u) {
    if (vis[u]) {
        if (vis[u] == 1) {
            cycs.emplace_back(find(begin(st), end(st), u), st.end());
            for (auto& a : cycs.back()) {
                cyc[a] = true;
            }
        }
        return;
    }
    vis[u] = 1;
    st.push_back(u);
    for (auto& v : graph[u]) {
        dfs(v);
    }
    vis[u] = 2;
    st.pop_back();
}
void dfs1(int u, int p, int d) {
    depth[u] = d;
    r_par[u] = p;
    for (auto& v : graph[u]) {
        if (cyc[v]) {
            continue;
        }
        dfs1(v, p, d + 1);
    }
}
void dfs2(int u, int d, int mod) {
    ans[u] = ans_add + d;
    for (auto& v : graph[u]) {
        if (cyc[v]) {
            continue;
        }
        dfs2(v, (d + 1) % mod, mod);
    }
}
void solve() {
    int n;
    cin >> n;
    int par[n];
    for (auto& a : par) {
        cin >> a;
        a--;
    }
    for (int i = 0; i < n; i++) {
        graph[par[i]].push_back(i);
    }
    for (int i = 0; i < n; i++) {
        dfs(i);
    }
    for (int i = 0; i < n; i++) {
        if (cyc[i]) {
            dfs1(i, i, -1);
        }
    }
    pair<int, int> opt {0, 0};
    for (int i = 0; i < n; i++) {
        opt = max(opt, {depth[i], i});
    }
    int u_ind = -1, u_nv = opt.second;
    for (int i = 0; i < n; i++) {
        if (cyc[i] && par[i] == r_par[opt.second]) {
            u_ind = i;
        }
    }
    par[u_ind] = u_nv;
    cycs.clear();
    for (int i = 0; i < n; i++) {
        vis[i] = 0;
        cyc[i] = false;
        graph[i].clear();
    }
    for (int i = 0; i < n; i++) {
        graph[par[i]].push_back(i);
    }
    for (int i = 0; i < n; i++) {
        dfs(i);
    }
    for (auto& a : cycs) {
        for (int i = 0; i < sz(a); i++) {
            dfs2(a[i], i, sz(a));
        }
        ans_add += sz(a);
    }
    cout << u_ind + 1 << " " << u_nv + 1 << endl;
    for (int i = 0; i < n; i++) {
        cout << ans[i] + 1 << " \n"[i == n - 1];
    }
}
