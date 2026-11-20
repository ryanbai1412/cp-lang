
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
    vector<int> depth, tin, tout, ord, par;
    vector<long> psum;
    vector<pair<int, int>> ord2;
    vector<vector<pair<int, int>>> graph;
    ST1<pair<int, int>> st_ord;
    Tree() {}
    Tree(const vector<array<int, 3>>& edges)
        : n(sz(edges) + 1),
          depth(n),
          tin(n),
          tout(n),
          par(n),
          psum(n),
          graph(n) {
        for (auto& [u, v, w] : edges) {
            graph[u].emplace_back(v, w);
        }
        dfs(0, -1, 0, 0);
        st_ord = ord2;
    }
    void dfs(int u, int p, int d, long ps) {
        par[u] = p;
        depth[u] = d;
        psum[u] = ps;
        tin[u] = sz(ord);
        for (int i = 0; i < sz(graph[u]); i++) {
            auto& [v, w] = graph[u][i];
            ord.push_back(u);
            ord2.emplace_back(tin[u], -i);
            dfs(v, u, d + 1, ps + w);
        }
        ord.push_back(u);
        ord2.emplace_back(tin[u], 1);
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
    int im_ch2(int u, int v) const {
        if (u == v) {
            return u;
        }
        return im_ch(u, v);
    }
    long dist_w(int u, int v) const {
        int l = lca(u, v);
        return psum[u] + psum[v] - 2 * psum[l];
    }
} tree;
struct DS1 {
    unordered_map<int, long> mp;
    multiset<pair<long, int>> ms;
    void insert(int x, long y) {
        auto [it, inserted] = mp.emplace(x, y);
        if (inserted) {
            ms.emplace(y, x);
            return;
        }
        if (y > it->second) {
            ms.erase({it->second, x});
            it->second = y;
            ms.emplace(y, x);
        }
    }
    void erase_key(int x) {
        auto it = mp.find(x);
        if (it == mp.end()) {
            return;
        }
        ms.erase({it->second, x});
        mp.erase(it);
    }
    long query_mx(int x) const {
        if (!sz(ms)) {
            return -1e18;
        }
        auto [cv, cx] = *(--end(ms));
        if (cx != x) {
            return cv;
        }
        if (sz(ms) == 1) {
            return -1e18;
        }
        return (--(--end(ms)))->first;
    }
};
constexpr int MAXN = 1e5 + 5;
long ans;
vector<tuple<int, int, long>> arr;
vector<tuple<int, long, long>> arr_w;
vector<vector<pair<int, int>>> arr_evts;
int gid = 0;
struct DS3 {
    using P = pair<int, long>;
    static long query(const P& a, const P& b) {
        auto& [au, aw] = a;
        auto& [bu, bw] = b;
        return tree.dist_w(au, bu) + aw + bw;
    }
    int id = ++gid;
    P p1 {0, long(-1e18)}, p2 {0, long(-1e18)};
    void insert(int u, long w) {
        // dbg(id, u, w);
        P cp {u, w};
        long copt = query(p1, p2), c1 = query(p1, cp), c2 = query(p2, cp);
        if (c2 < c1) {
            swap(c2, c1);
            swap(p2, p1);
        }
        if (c2 > copt) {
            p1 = cp;
        }
    }
    long query(int u, long w) const {
        P cp {u, w};
        return max(query(p1, cp), query(p2, cp));
    }
};
struct DS2 {
    DS1 ds1, ds2;
    vector<tuple<int, long, long>> hist;
    void insert(int k, long v1, long v2) {
        ds1.insert(k, v1);
        ds2.insert(k, v1 + v2);
        hist.emplace_back(k, v1, v2);
    }
    void erase_key(int k) {
        ds1.erase_key(k);
        ds2.erase_key(k);
    }
    long query(int k, long v1, long v2) const {
        return max(ds1.query_mx(k) + v1 + v2, ds2.query_mx(k) + v1);
    }
    void insert(tuple<int, long, long> v) {
        auto& [k, v1, v2] = v;
        insert(k, v1, v2);
    }
    long query(tuple<int, long, long> v) const {
        auto& [k, v1, v2] = v;
        return query(k, v1, v2);
    }
    int size() const {
        return sz(ds1.mp) + sz(ds2.mp) + sz(hist2) + sz(hist3);
    }
    unordered_map<int, DS3> lca2;
    vector<tuple<int, int, long>> hist2;
    void insert2(int p, int v, long w) {
        hist2.emplace_back(p, v, w);
        lca2[p].insert(v, w);
    }
    long query2(int p, int v, long w) {
        return lca2[p].query(v, w);
    }
    unordered_map<int, DS1> lca3;
    vector<tuple<int, int, long>> hist3;
    void insert3(int p, int v, long w) {
        hist3.emplace_back(p, v, w);
        lca3[p].insert(v, w);
    }
    long query3(int p, int v) {
        return lca3[p].query_mx(v);
    }
};
DS2 solve1(int u) {
    DS2 cds;
    for (auto& [v, _w] : tree.graph[u]) {
        auto&& nds = solve1(v);
        if (sz(nds) > sz(cds)) {
            swap(cds, nds);
        }
        for (auto& [k, v1, v2] : nds.hist) {
            if (tree.anc(u, k)) {
                continue;
            }
            ans = max(ans, cds.query(k, v1, v2) - tree.psum[u]);
        }
        for (auto& [k, v1, v2] : nds.hist) {
            if (tree.anc(u, k)) {
                continue;
            }
            cds.insert(k, v1, v2);
        }
        for (auto& [p, v, w] : nds.hist2) {
            if (tree.depth[p] <= tree.depth[u]) {
                ans = max(ans, (cds.query2(p, v, w) - 2 * tree.psum[u]) / 2);
            }
        }
        for (auto& [p, v, w] : nds.hist2) {
            if (tree.depth[p] <= tree.depth[u]) {
                cds.insert2(p, v, w);
            }
        }
        for (auto& [p, v, w] : nds.hist3) {
            if (tree.depth[p] < tree.depth[u]) {
                ans = max(ans, cds.query3(p, v) + w - tree.dist_w(u, p));
            }
        }
        for (auto& [p, v, w] : nds.hist3) {
            if (tree.depth[p] < tree.depth[u]) {
                cds.insert3(p, v, w);
            }
        }
    }
    for (auto& [ei, ty] : arr_evts[u]) {
        if (ty != +1) {
            continue;
        }
        ans = max(ans, cds.query(arr_w[ei]) - tree.psum[u]);
        cds.insert(arr_w[ei]);
        {
            auto& [eu, ev, ew] = arr[ei];
            int lca = tree.lca(eu, ev);
            assert(u == eu || u == ev);
            int other = eu ^ ev ^ u;
            dbg(lca, other);
            static int xid = -1;
            int qp = lca, qv;
            if (other == qp) {
                qv = --xid;
            } else {
                qv = tree.im_ch(lca, other);
            }
            long qw = tree.dist_w(eu, ev) - ew;
            dbg(u, qp, qv, qw);
            ans = max(ans, cds.query3(qp, qv) + qw - tree.dist_w(u, qp));
            cds.insert3(qp, qv, qw);
        }
        {
            auto& [eu, ev, ew] = arr[ei];
            int lca = tree.lca(eu, ev);
            if (eu == lca || ev == lca) {
                continue;
            }
            assert(u == eu || u == ev);
            int other = eu ^ ev ^ u;
            dbg(lca, other);
            int qp = tree.im_ch(lca, other), qv = other;
            long qw = tree.dist_w(eu, ev) - 2 * ew + tree.psum[u];
            assert(cds.query2(qp, qv, qw) % 2 == 0);
            if (ei == 2 && u == 1) {
                dbg(qp, qv, qw, cds.query2(qp, qv, qw));
                dbg(cds.lca2[qp].hist);
            }
            ans = max(ans, (cds.query2(qp, qv, qw) - 2 * tree.psum[u]) / 2);
            cds.insert2(qp, qv, qw);
        }
    }
    cds.erase_key(tree.par[u]);
    return cds;
}
void dfs(const vector<vector<pair<int, int>>>& graph,
         vector<int>& st,
         vector<int>& ans,
         int u,
         int targ,
         int p) {
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
    bool line = true;
    vector<array<int, 3>> edges(n - 1);
    for (auto& [u, v, w] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
        line &= u + 1 == v;
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
    if (false && n <= 200 && m <= 300) {
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
            arr_evts[tree.im_ch(lca, cx)].emplace_back(i, -1);
        } else {
            arr_evts[u].emplace_back(i, +1);
            arr_evts[v].emplace_back(i, +1);
            arr_evts[tree.im_ch(lca, u)].emplace_back(i, -1);
            arr_evts[tree.im_ch(lca, v)].emplace_back(i, -1);
            // arr_evts[lca].emplace_back(i, -2);
        }
        arr_w[i] = {lca, tree.dist_w(u, v) - w, +tree.psum[lca]};
    }
    ans = -1e18;
    solve1(0);
    dbg(ans);
    if (ans <= long(-1e17)) {
        cout << "F" << endl;
    } else {
        cout << ans << endl;
    }
}
