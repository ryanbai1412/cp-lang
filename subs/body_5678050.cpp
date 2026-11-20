
constexpr int maxn = 2e5 + 5;
struct DS {
    int cnt[maxn] {};
    set<pair<int, int>> s;
    DS() {
        for (int i = 0; i < maxn; i++) {
            s.insert(g_pair(i));
        }
    }
    pair<int, int> g_pair(int ind) const {
        return {cnt[ind], -ind};
    }
    void update(int ind, int x) {
        s.erase(g_pair(ind));
        cnt[ind] += x;
        s.insert(g_pair(ind));
    }
    int query_all() const {
        return -s.rbegin()->second;
    }
} ds;
int arr[maxn][2], ans[maxn];
vector<int> graph[maxn];
void dfs(int u, int pt) {
    if (u == -1) {
        return;
    }
    ds.update(arr[u][0], pt - u);
    dbg(ds.query_all());
    ans[ds.query_all()]++;
    for (auto& v : graph[u]) {
        dfs(v, u);
    }
    ds.update(arr[u][0], -(pt - u));
}
void solve() {
    int n, m;
    cin >> n >> m;
    int rep[n];
    memset(rep, -1, sizeof(rep));
    for (int i = 0; i < m; i++) {
        auto& [u, v] = arr[i];
        cin >> u >> v;
        graph[i].push_back(rep[u]);
        graph[i].push_back(rep[v]);
        rep[u] = i;
        rep[v] = -1;
    }
    for (int i = 0; i < n; i++) {
        dfs(rep[i], m);
    }
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}
