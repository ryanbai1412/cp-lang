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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T, typename Cmp = less<T>>
void sort_unique(vector<T>& arr, Cmp cmp = {}) {
    sort(begin(arr), end(arr), cmp);
    arr.erase(unique(begin(arr), end(arr)), arr.end());
}

template <typename T>
struct ST {
    int n;
    vector<vector<T>> v;

    ST() {}
    ST(const vector<T>& arr) : n(sz(arr)), v(__lg(n) + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= __lg(n); i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = min(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    T query(int l, int r) const {
        int k = __lg(r - l);
        return min(v[k][l], v[k][r - (1 << k)]);
    }
};

struct Tree {
    int n;
    vector<int> tin, tout, depth, ord;
    vector<vector<int>> graph;
    ST<int> st_ord;

    Tree(int root, const vector<pair<int, int>>& edges)
        : n(sz(edges) + 1), tin(n), tout(n), depth(n), graph(n) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        dfs(root, -1, 0);
        st_ord = (({
            vector<int> v;
            for (auto& a : ord) {
                v.push_back(tin[a]);
            }
            v;
        }));
    }

    void dfs(int u, int p, int d) {
        tin[u] = sz(ord);
        depth[u] = d;
        ord.push_back(u);
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u, d + 1);
            ord.push_back(u);
        }
        tout[u] = sz(ord);
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }

    int lca(int u, int v) const {
        if (tin[u] > tin[v]) {
            swap(u, v);
        }
        return ord[st_ord.query(tin[u], tout[v])];
    }

    struct VirtualTree {
        vector<int> nodes;
        vector<pair<int, int>> edges;
    };

    VirtualTree virtual_tree(vector<int> nodes) const {
        auto tin_cmp = CmpByKey([&](int u) -> int { return tin[u]; });
        sort(begin(nodes), end(nodes), tin_cmp);

        vector<int> a_nodes = nodes;
        for (int i = 0; i < sz(nodes) - 1; i++) {
            a_nodes.push_back(lca(nodes[i], nodes[i + 1]));
        }
        sort_unique(a_nodes, tin_cmp);

        vector<pair<int, int>> edges;

        vector<int> st {a_nodes[0]};
        for (int i = 1; i < sz(a_nodes); i++) {
            int u = a_nodes[i];
            while (!anc(st.back(), u)) {
                st.pop_back();
            }

            edges.emplace_back(st.back(), u);
            st.push_back(u);
        }

        return {a_nodes, edges};
    }
};
struct SolveTree {
    int n;
    Tree tree;
    vector<char> vis1, vis2;
    vector<vector<pair<int, int>>> graph, igraph;

    SolveTree(int root, const vector<pair<int, int>>& edges)
        : n(sz(edges) + 1),
          tree(root, edges),
          vis1(n),
          vis2(n),
          graph(n),
          igraph(n) {}

    template <typename Cb>
    void dfs(int u,
             vector<char>& vis,
             const vector<vector<pair<int, int>>>& graph,
             const Cb& cb) {
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        for (auto& [v, ei] : graph[u]) {
            cb(ei);
            dfs(v, vis, graph, cb);
        }
    }

    int query(int src, int dest, const vector<pair<int, int>>& q_edges) {
        auto [nodes, edges] = tree.virtual_tree(({
            vector<int> cn {src, dest};
            for (auto& [u, v] : q_edges) {
                cn.push_back(u);
                cn.push_back(v);
            }
            cn;
        }));
        dbg(nodes, edges);

        for (auto& a : nodes) {
            graph[a].clear();
            igraph[a].clear();
            vis1[a] = vis2[a] = false;
        }

        int m = sz(edges);

        auto add_edge = [&](int ei, int u, int v) -> void {
            graph[u].emplace_back(v, ei);
            igraph[v].emplace_back(u, ei);
        };

        for (int i = 0; i < m; i++) {
            auto& [u, v] = edges[i];
            add_edge(i, u, v);
        }
        for (int i = 0; i < sz(q_edges); i++) {
            auto& [u, v] = q_edges[i];
            add_edge(m + i, u, v);
        }

        int cnt_edge[m] {};

        auto cb_edge = [&](int ei) -> void {
            if (ei >= m) {
                return;
            }
            cnt_edge[ei]++;
        };

        dfs(src, vis1, graph, cb_edge);
        dfs(dest, vis2, igraph, cb_edge);

        int ans = 0;
        for (auto& a : nodes) {
            ans += vis1[a] && vis2[a];
        }
        for (int i = 0; i < m; i++) {
            auto& [u, v] = edges[i];
            if (cnt_edge[i] == 2) {
                ans += tree.depth[v] - tree.depth[u] - 1;
            }
        }

        return ans;
    }
};

void solve() {
    int n, m, q, kv;
    cin >> n >> m >> q >> kv;
    vector<pair<int, int>> edges(m);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }

    int i_deg[n] {};
    for (auto& [u, v] : edges) {
        i_deg[v]++;
    }
    int root = int(find(i_deg, i_deg + n, 0) - i_deg);

    SolveTree solver(root, edges);

    while (q--) {
        int src, dest;
        vector<pair<int, int>> q_edges(kv);
        cin >> src >> dest;
        src--;
        dest--;
        for (auto& [u, v] : q_edges) {
            cin >> u >> v;
            u--;
            v--;
        }
        cout << solver.query(src, dest, q_edges) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}