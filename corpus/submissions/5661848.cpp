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
};
 
vector<pair<int, int>> spanning_tree(int n,
                                     const vector<pair<int, int>>& edges) {
    DSU dsu(n);
    vector<pair<int, int>> ans;
    for (auto& [u, v] : edges) {
        if (dsu.merge(u, v)) {
            ans.emplace_back(u, v);
        }
    }
    return ans;
}
 
struct Centroid {
    int n, cu;
    vector<int> siz;
    vector<vector<int>> graph, comps;
 
    Centroid(int n, const vector<pair<int, int>>& edges)
        : n(n), siz(n), graph(n) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        pdfs(0, -1);
 
        cu = 0;
        while (true) {
            pair<int, int> opt {-1, -1};
            for (auto& v : graph[cu]) {
                opt = max(opt, {siz[v], v});
            }
            if (opt.first <= n / 2) {
                break;
            }
            int nu = opt.second;
            siz[cu] -= siz[nu];
            siz[nu] += siz[cu];
            cu = nu;
        }
 
        for (auto& v : graph[cu]) {
            comps.emplace_back();
            dfs(v, cu, comps.back());
            dbg(v, siz[v], sz(comps.back()));
        }
    }
 
    void pdfs(int u, int p) {
        siz[u] = 1;
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            pdfs(v, u);
            siz[u] += siz[v];
        }
    }
 
    void dfs(int u, int p, vector<int>& out) {
        out.push_back(u);
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u, out);
        }
    }
};
 
struct Solver1 {
    bool found = false;
    int n, kv, csum;
    vector<char> vis;
    vector<int> arr, st;
    vector<vector<int>> graph;
 
    Solver1(int n,
            int kv,
            const vector<int>& arr,
            const vector<pair<int, int>>& edges)
        : n(n), kv(kv), vis(n), arr(arr), graph(n) {
        assert(*max_element(begin(arr), end(arr)) < kv);
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        for (int i = 0; i < n && !found; i++) {
            if (vis[i]) {
                continue;
            }
            st.clear();
            csum = 0;
            dfs(i);
            found |= csum >= kv;
        }
    }
 
    void dfs(int u) {
        if (found || vis[u]) {
            return;
        } else if (csum >= kv) {
            assert(csum <= 2 * kv - 2);
            found = true;
            return;
        }
        vis[u] = true;
        st.push_back(u);
        csum += arr[u];
        for (auto& v : graph[u]) {
            dfs(v);
        }
    }
};
 
struct Solver2 {
    int n, kv;
    vector<char> vis;
    vector<int> st;
    vector<vector<int>> graph;
 
    Solver2(int n,
            int kv,
            const vector<int>& nodes,
            const vector<pair<int, int>>& edges)
        : n(n), kv(kv), vis(n), graph(n) {
        bool inode[n] {};
        for (auto& a : nodes) {
            inode[a] = true;
        }
 
        assert(kv <= n);
        for (auto& [u, v] : edges) {
            if (!inode[u] || !inode[v]) {
                continue;
            }
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
 
        dfs(nodes[0]);
 
        assert(sz(st) == kv);
    }
 
    void dfs(int u) {
        if (vis[u] || sz(st) == kv) {
            return;
        }
        vis[u] = true;
        st.push_back(u);
        for (auto& v : graph[u]) {
            dfs(v);
        }
    }
};
 
vector<int> find_split(int n,
                       int kv1,
                       int kv2,
                       int kv3,
                       vector<int> edges_u,
                       vector<int> edges_v) {
    int m = sz(edges_u);
 
    vector<pair<int, int>> edges(m);
    for (int i = 0; i < m; i++) {
        edges[i] = {edges_u[i], edges_v[i]};
    }
 
    array<int, 3> ikv {kv1, kv2, kv3};
    {
        array<int, 3> ckv {kv1, kv2, kv3};
        sort(begin(ckv), end(ckv));
        kv1 = ckv[0];
        kv2 = ckv[1];
        kv3 = ckv[2];
    }
 
    auto repermute = [&](vector<int> arr) -> vector<int> {
        int cnt[3] {};
        for (auto& a : arr) {
            cnt[a - 1]++;
        }
 
        int perm[3];
        iota(begin(perm), end(perm), 0);
 
        do {
            bool ok = true;
            for (int i = 0; i < 3; i++) {
                ok &= ikv[perm[i]] == cnt[i];
            }
            if (!ok) {
                continue;
            }
 
            for (auto& a : arr) {
                a = perm[a - 1] + 1;
            }
            return arr;
        } while (next_permutation(begin(perm), end(perm)));
 
        assert(false);
    };
 
    dbg(kv1, kv2, kv3);
 
    Centroid centroid(n, spanning_tree(n, edges));
    auto& comps = centroid.comps;
 
    int i_comp[n];
    i_comp[centroid.cu] = -1;
    for (int i = 0; i < sz(comps); i++) {
        for (auto& a : comps[i]) {
            i_comp[a] = i;
        }
    }
 
    auto not_nodes = [&](const vector<int>& arr) -> vector<int> {
        bool vis[n] {};
        for (auto& a : arr) {
            vis[a] = true;
        }
        vector<int> ans;
        for (int i = 0; i < n; i++) {
            if (!vis[i]) {
                ans.push_back(i);
            }
        }
        return ans;
    };
    auto answer = [&](const vector<int>& arra,
                      const vector<int>& arrb) -> vector<int> {
        dbg(sz(arra), sz(arrb));
        assert(kv1 <= sz(arra) && kv2 <= sz(arrb));
        vector<int> ans(n, 3);
 
        auto go = [&](const vector<int>& nodes, int kv, int val) -> void {
            auto c_comp = Solver2(n, kv, nodes, edges).st;
            for (auto& a : c_comp) {
                ans[a] = val;
            }
        };
 
        go(arra, kv1, 1);
        go(arrb, kv2, 2);
 
        return repermute(ans);
    };
 
    for (int i = 0; i < sz(comps); i++) {
        if (sz(comps[i]) < kv1) {
            continue;
        }
 
        auto arra = comps[i];
        dbg("centroid quick");
        return answer(arra, not_nodes(arra));
    }
 
    vector<int> c_sizes;
    for (auto& a : comps) {
        c_sizes.push_back(sz(a));
    }
    vector<pair<int, int>> c_edges;
    for (auto& [u, v] : edges) {
        if (u == centroid.cu || v == centroid.cu) {
            continue;
        }
        dbg(i_comp[u], i_comp[v]);
        c_edges.emplace_back(i_comp[u], i_comp[v]);
    }
 
    Solver1 s1(sz(comps), kv1, c_sizes, c_edges);
 
    if (!s1.found) {
        // assert(false);
        return vector<int>(n);
    }
 
    vector<int> arra;
    for (auto& a : s1.st) {
        arra.insert(arra.end(), begin(comps[a]), end(comps[a]));
    }
    return answer(arra, not_nodes(arra));
}