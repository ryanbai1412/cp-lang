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

struct Graph {
    int n, m;
    vector<int> el;
    vector<pair<int, int>> edges;

    Graph(int n, const vector<pair<int, int>>& _edges)
        : n(n), el(n), edges(_edges) {
        sort(begin(edges), end(edges));
        edges.erase(unique(begin(edges), end(edges)), edges.end());
        m = sz(edges);
        for (int i = 0, j = 0; i < n; i++) {
            el[i] = j;
            for (; j < m && edges[j].first == i; j++)
                ;
        }
    }

    template <typename Cb>
    void adj(int u, const Cb& cb) const {
        for (int i = el[u]; i < m && edges[i].first == u; i++) {
            cb(edges[i].second);
        }
    }

    Graph inverse() const {
        vector<pair<int, int>> nedges = edges;
        for (auto& [u, v] : nedges) {
            swap(u, v);
        }
        return Graph(n, nedges);
    }
};

struct SCC {
    static void dfs(int u,
                    const Graph& graph,
                    vector<bool>& vis,
                    vector<int>& ord) {
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        graph.adj(u, [&](int v) -> void { dfs(v, graph, vis, ord); });
        ord.push_back(u);
    }

    int n, cind = 0;
    vector<int> comp_ind;

    SCC(const Graph& graph)
        : n(graph.n), comp_ind(n) {
        Graph igraph = graph.inverse();
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
            if (vis[a]) {
                continue;
            }
            vector<int> comp;
            dfs(a, igraph, vis, comp);
            for (auto& b : comp) {
                comp_ind[b] = cind;
            }
            cind++;
        }
    }
};

struct Solver {
    int n;
    long xans;
    vector<int> arr;
    vector<long> memo;
    vector<bool> pub;
    const Graph& graph;

    Solver(int src,
           const vector<int>& arr,
           const vector<bool>& pub,
           const Graph& graph)
        : n(sz(arr)), arr(arr), memo(n, -1), pub(pub), graph(graph) {
        xans = dp(src);
    }

    long dp(int u) {
        long& ans = memo[u];
        if (ans != -1) {
            return ans;
        }
        ans = -1e18;
        if (pub[u]) {
            ans = 0;
        }
        graph.adj(u, [&](int v) -> void { ans = max(ans, dp(v)); });
        ans += arr[u];
        return ans;
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    vector<pair<int, int>> edges(m);
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
    Graph graph(n, edges);
    SCC scc(graph);
    auto& comp_ind = scc.comp_ind;
    int cn = scc.cind;
    vector<int> carr(cn);
    vector<bool> pub(cn);
    Graph cgraph = Graph(n, ({
                             vector<pair<int, int>> cedges;
                             for (auto& [u, v] : edges) {
                                 int cu = comp_ind[u], cv = comp_ind[v];
                                 if (cu != cv) {
                                     cedges.emplace_back(cu, cv);
                                 }
                             }
                             cedges;
                         }));
    for (int i = 0; i < n; i++) {
        carr[comp_ind[i]] += arr[i];
    }
    for (auto& a : pubs) {
        assert(a >= 0);
        pub[comp_ind[a]] = true;
    }
    dbg(pub[1]);
    cout << Solver(comp_ind[src], carr, pub, cgraph).xans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}