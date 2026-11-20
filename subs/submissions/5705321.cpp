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
struct RMQ {
    int n;
    vector<vector<T>> v;

    RMQ() {}
    RMQ(const vector<T>& arr) : n(sz(arr)), v(__lg(n) + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i < __lg(n); i++) {
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
    int n, logn;
    vector<int> par, tin, tout, ord;
    vector<vector<int>> graph, lift;

    Tree(int n, const vector<pair<int, int>>& edges)
        : n(n),
          logn(__lg(n)),
          par(n),
          tin(n),
          tout(n),
          graph(n),
          lift(logn + 1, vector<int>(n)) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        dfs(0, -1);

        lift[0] = par;
        for (int i = 1; i <= logn; i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }

    void dfs(int u, int p) {
        tin[u] = sz(ord);
        par[u] = p;

        ord.push_back(u);

        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }

            dfs(v, u);
            ord.push_back(u);
        }

        tout[u] = sz(ord);
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }

    template <typename Cb>
    int first_anc(int u, const Cb& cb) const {
        if (cb(u)) {
            return u;
        }

        for (int i = logn; i >= 0; i--) {
            int nu = lift[i][u];
            if (nu != -1 && !cb(nu)) {
                u = nu;
            }
        }

        return par[u];
    }

    int par_child(int u, int v) const {
        assert(anc(u, v) && u != v);

        return first_anc(
            v, [&](int cv) -> bool { return anc(cv, u) || anc(par[cv], u); });
    }
};

struct Nodes {
    const Tree& tree;
    set<int> nodes, tins;

    Nodes(const Tree& tree) : tree(tree) {
        for (int i = 0; i < tree.n; i++) {
            insert(i);
        }
    }

    void insert(int u) {
        nodes.insert(u);
        tins.insert(tree.tin[u]);
    }

    void erase(int u) {
        nodes.erase(u);
        tins.erase(tree.tin[u]);
    }

    int main_anc() const {
        assert(sz(tins));
        return tree.ord[*tins.begin()];
    }

    bool has_subtree(int u) const {
        return tins.lower_bound(tree.tin[u]) != tins.lower_bound(tree.tout[u]);
    }

    int size() const {
        return sz(nodes);
    }

    pair<int, int> closest_node(int u) const {
        assert(!nodes.count(u));

        int m_anc = main_anc();
        {
            if (tree.anc(u, m_anc)) {
                return {m_anc, tree.par[m_anc]};
            }
        }

        if (!tree.anc(m_anc, u)) {
            return {m_anc, tree.par[m_anc]};
        }

        int lca =
            tree.first_anc(u, [&](int v) -> bool { return has_subtree(v); });

        assert(nodes.count(lca));

        return {lca, tree.par_child(lca, u)};
    }
};

void solve() {
    int n, m;
    cin >> n >> m;

    vector<pair<int, int>> edges(n - 1);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }

    int arr[m];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    Tree tree(n, edges);
    Nodes nodes(tree);

    set<int> graph[n];
    for (auto& [u, v] : edges) {
        graph[u].insert(v);
        graph[v].insert(u);
    }

    set<int> leaves;
    auto upd_leaf = [&](int u) -> void {
        if (sz(graph[u]) == 1) {
            leaves.insert(u);
        } else {
            leaves.erase(u);
        }
    };

    for (int i = 0; i < n; i++) {
        upd_leaf(i);
    }

    for (auto& u : arr) {
        if (sz(nodes) == 1) {
            cout << 1 << endl;
            continue;
        }

        vector<int> to_erase(begin(leaves), end(leaves));

        optional<pair<int, int>> to_insert;

        if (nodes.nodes.count(u)) {
            if (leaves.count(u)) {
                to_erase.erase(find(begin(to_erase), end(to_erase), u));
            }
        } else {
            auto [c1, c2] = nodes.closest_node(u);

            to_erase.erase(find(begin(to_erase), end(to_erase), c1));
            to_insert = {c1, c2};
        }

        for (auto& a : to_erase) {
            nodes.erase(a);

            for (auto& v : graph[a]) {
                graph[v].erase(a);
                upd_leaf(v);
            }
            graph[a].clear();
            upd_leaf(a);
        }

        if (to_insert) {
            auto [c1, c2] = to_insert.value();

            assert(nodes.nodes.count(c1) && !nodes.nodes.count(c2));

            nodes.insert(c2);

            graph[c1].insert(c2);
            graph[c2].insert(c1);

            upd_leaf(c1);
            upd_leaf(c2);
        }

        cout << sz(nodes) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}