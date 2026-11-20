#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(T&& t, U&&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

constexpr int maxn = 5e5 + 5;

struct SCC {
    static void dfs(int u,
                    const vector<vector<int>>& graph,
                    vector<bool>& vis,
                    vector<int>& ord) {
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        for (auto& v : graph[u]) {
            dfs(v, graph, vis, ord);
        }
        ord.push_back(u);
    }

    int n;
    vector<vector<int>> comps, graph, igraph;

    SCC(const vector<vector<int>>& graph)
        : n(sz(graph)), graph(graph), igraph(n) {
        for (int i = 0; i < n; i++) {
            for (auto& v : graph[i]) {
                igraph[v].push_back(i);
            }
        }
        vector<bool> vis(n);
        vector<int> ord;
        for (int i = 0; i < n; i++) {
            if (vis[i]) {
                continue;
            }
            dfs(i, graph, vis, ord);
        }
        fill(begin(vis), end(vis), false);
        reverse(begin(ord), end(ord));
        for (auto& a : ord) {
            dbg(a + 1);
            if (vis[a]) {
                continue;
            }
            vector<int> comp;
            dfs(a, igraph, vis, comp);
            comps.push_back(comp);
        }
    }
};

struct Solver {
    int n;
    long xans;
    vector<int> arr;
    vector<long> memo;
    vector<bool> pub;
    vector<vector<int>> graph;

    Solver(int src, const vector<int>& arr,
           const vector<bool>& pub,
           const vector<vector<int>>& graph)
        : n(sz(arr)), arr(arr), memo(n, -1), pub(pub), graph(graph) {
        xans = dp(src);
    }

    long dp(int u) {
        long& ans = memo[u];
        if (ans != -1) {
            return ans;
        }
        ans = 1e18;
        if (pub[u]) {
            ans = 0;
        }
        for (auto& v : graph[u]) {
            ans = max(ans, dp(v));
        }
        ans += arr[u];
        return ans;
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    int edges[m][2];
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int src, kv;
    cin >> src >> kv;
    src--;
    int pubs[kv];
    for (auto& a : pubs) {
        cin >> a;
        a--;
    }
    vector<vector<int>> graph(n);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
    }
    auto comps = SCC(graph).comps;
    for (auto& a : comps) {
        for (auto& b : a) {
            cerr << b + 1 << " ";
        }
        cerr << endl;
    }
    int comp_ind[n];
    for (int i = 0; i < sz(comps); i++) {
        for (auto& a : comps[i]) {
            comp_ind[a] = i;
        }
    }
    int cn = sz(comps);
    vector<int> carr(cn);
    vector<bool> pub(cn);
    vector<vector<int>> cgraph(cn);
    for (auto& [u, v] : edges) {
        int cu = comp_ind[u], cv = comp_ind[v];
        if (cu != cv) {
            cgraph[cu].push_back(cv);
        }
    }
    for (int i = 0; i < n; i++) {
        carr[comp_ind[i]] += arr[i];
    }
    for (auto& a : pubs) {
        pub[comp_ind[a]] = true;
    }
    cout << Solver(src, carr, pub, cgraph).xans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}