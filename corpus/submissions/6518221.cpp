#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
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

template <typename T>
struct ST1 {
    int n, logn;
    vector<vector<T>> v;

    ST1() {}
    ST1(const vector<T>& arr)
        : n(sz(arr)), logn(__lg(n + 10)), v(logn + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= logn; i++) {
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
    vector<int> depth, tin, tout, ord;
    vector<long> psum;
    vector<pair<int, int>> ord2;
    vector<vector<pair<int, int>>> graph;
    ST1<pair<int, int>> st_ord;

    Tree() {}
    Tree(const vector<array<int, 3>>& edges)
        : n(sz(edges) + 1), depth(n), tin(n), tout(n), psum(n), graph(n) {
        for (auto& [u, v, w] : edges) {
            graph[u].emplace_back(v, w);
        }
        dfs(0, 0, 0);
        st_ord = ord2;
    }

    void dfs(int u, int d, long ps) {
        depth[u] = d;
        psum[u] = ps;

        tin[u] = sz(ord);

        for (int i = 0; i < sz(graph[u]); i++) {
            auto& [v, w] = graph[u][i];

            ord.push_back(u);
            ord2.emplace_back(tin[u], -i);
            dfs(v, d + 1, ps + w);
        }

        tout[u] = sz(ord);
    }

    int lca(int u, int v) const {
        if (tin[u] > tin[v]) {
            swap(u, v);
        }
        int ans = ord[st_ord.query(tin[u], tout[v]).first];
        assert(anc(ans, u) && anc(ans, v));
        return ans;
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }

    int im_ch(int u, int v) const {
        assert(anc(u, v) && u != v);

        int ans = graph[u][-st_ord.query(tin[u], tin[v]).second].first;
        assert(anc(ans, v));
        return ans;
    }

    long dist_w(int u, int v) const {
        int l = lca(u, v);

        return psum[u] + psum[v] - 2 * psum[l];
    }
} tree;

struct DS1 {
    map<int, multiset<long>> mp;
    multiset<pair<int, long>> ms;

    static long g_mx(multiset<long>& s) {
        if (!sz(s)) {
            return -1e18;
        }
        return *(--end(s));
    }

    void insert(int x, int y) {
        auto& cs = mp[x];

        auto it = ms.find({x, g_mx(cs)});
        if (it != end(ms)) {
            ms.erase(it);
        }

        cs.insert(y);

        ms.emplace(x, g_mx(cs));
    }

    void erase(int x, int y) {
        auto& cs = mp[x];

        auto it = ms.find({x, g_mx(cs)});
        if (it != end(ms)) {
            ms.erase(it);
        }

        cs.erase(y);

        ms.emplace(x, g_mx(cs));
    }

    long query_mx(int x) const {
        if (!sz(ms)) {
            return -1e18;
        }
        auto [cx, cv] = *(--end(ms));
        if (cx != x) {
            return cv;
        }

        if (sz(ms) == 1) {
            return -1e18;
        }
        return (--(--end(ms)))->second;
    }
};

constexpr int MAXN = 1e5 + 5;

long ans;
vector<tuple<int, int, long>> arr;
vector<tuple<int, long, long>> arr_w;
vector<vector<pair<int, int>>> arr_evts;

struct DS2 {
    DS1 ds1, ds2;
    multiset<tuple<int, long, long>> hist;

    void insert(int k, long v1, long v2) {
        ds1.insert(k, v1);
        ds2.insert(k, v1 + v2);
        hist.emplace(k, v1, v2);
    }

    void erase(int k, long v1, long v2) {
        ds1.erase(k, v1);
        ds2.erase(k, v1 + v2);
        hist.erase({k, v1, v2});
    }

    long query(int k, long v1, long v2) const {
        return max(ds1.query_mx(k) + v1 + v2, ds2.query_mx(k) + v1);
    }

    void insert(tuple<int, long, long> v) {
        auto& [k, v1, v2] = v;
        insert(k, v1, v2);
    }
    void erase(tuple<int, long, long> v) {
        auto& [k, v1, v2] = v;
        erase(k, v1, v2);
    }
    long query(tuple<int, long, long> v) const {
        auto& [k, v1, v2] = v;
        return query(k, v1, v2);
    }

    int size() const {
        return sz(hist);
    }
};

DS2 solve1(int u) {
    dbg(u);
    DS2 cds;

    for (auto& [v, _w] : tree.graph[u]) {
        auto&& nds = solve1(v);

        if (sz(nds) > sz(cds)) {
            swap(cds, nds);
        }

        for (auto& [k, v1, v2] : nds.hist) {
            ans = max(ans, cds.query(k, v1, v2) - tree.psum[u]);
        }
        for (auto& [k, v1, v2] : nds.hist) {
            cds.insert(k, v1, v2);
        }
    }

    for (auto& [ei, ty] : arr_evts[u]) {
        if (ty != +1) {
            continue;
        }

        ans = max(ans, cds.query(arr_w[ei]) - tree.psum[u]);
        cds.insert(arr_w[ei]);
    }

    for (auto& [ei, ty] : arr_evts[u]) {
        if (ty >= 0) {
            continue;
        }

        for (int it = 0; it < -ty; it++) {
            cds.erase(arr_w[ei]);
        }
    }

    return cds;
}

void dfs(const vector<vector<pair<int, int>>>& graph, vector<int>& st, vector<int>& ans, int u, int targ, int p) {
    if (u == targ) {
        ans = st;
        return;
    }

    for (auto& [v, ei] : graph[u]) {
        if (v == p) {
            continue;
        }
        st.push_back(ei);

        dfs(graph, st, ans, v, targ, u);

        st.pop_back();
    }
}

void solve() {
    int n;
    cin >> n;

    vector<array<int, 3>> edges(n - 1);
    for (auto& [u, v, w] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
    }

    int m;
    cin >> m;
    arr.clear();
    for (int i = 0; i < m; i++) {
        int u, v;
        long w;
        cin >> u >> v >> w;
        u--;
        v--;

        if (u == v) {
            continue;
        }

        arr.emplace_back(u, v, w);
    }
    m = sz(arr);

    tree = Tree(edges);

    if (n <= 200 && m <= 300) {
        vector<vector<pair<int, int>>> graph(n);

        for (int i = 0; i < n - 1; i++) {
            auto& [u, v, _w] = edges[i];
            graph[u].emplace_back(v, i);
            graph[v].emplace_back(u, i);
        }

        vector<int> st, dfs_ans;
        vector<vector<int>> arr_edge;

        for (int i = 0; i < m; i++) {
            auto& [u, v, _w] = arr[i];
            dfs(graph, st, dfs_ans, u, v, -1);
            arr_edge.push_back(dfs_ans);
        }

        ans = -1e18;

        for (int i = 0; i < m; i++) {
            for (int j = i + 1; j < m; j++) {
                int cnt[n - 1] {};
                for (auto& a : arr_edge[i]) {
                    cnt[a]++;
                }
                for (auto& a : arr_edge[j]) {
                    cnt[a]++;
                }

                if (*max_element(cnt, cnt + n - 1) < 2) {
                    continue;
                }

                long cans = get<2>(arr[i]) + get<2>(arr[j]);
                cans *= -1;
                for (int ei = 0; ei < n - 1; ei++) {
                    if (cnt[ei]) {
                        cans += get<2>(edges[ei]);
                    }
                }
                ans = max(ans, cans);
            }
        }

        if (ans <= long(-0.5e18)) {
            cout << "F" << endl;
        } else {
            cout << ans << endl;
        }

        return;
    }

    arr_evts.clear();
    arr_evts.resize(n);
    arr_w.resize(m);

    for (int i = 0; i < m; i++) {
        auto& [u, v, w] = arr[i];
        dbg(u, v);
        int lca = tree.lca(u, v);

        if (u == lca || v == lca) {
            int cx = u ^ v ^ lca;
            arr_evts[cx].emplace_back(i, +1);
            arr_evts[lca].emplace_back(i, -1);
        } else {
            arr_evts[u].emplace_back(i, +1);
            arr_evts[v].emplace_back(i, +1);
            arr_evts[lca].emplace_back(i, -2);
        }

        arr_w[i] = {lca, tree.dist_w(u, v) - w, +tree.psum[lca]};
    }

    ans = -1e18;

    solve1(0);

    dbg(ans);
    if (ans <= long(-0.5e18)) {
        cout << "F" << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}